use self::channel::Channel;
use self::pixel::Pixel;

mod channel {

    use core::slice::Iter;

    pub enum Channel {
        R,
        G,
        B,
    }

    impl Channel {
        pub fn to_number(&self) -> usize {
            match *self {
                Channel::R => 0,
                Channel::G => 1,
                Channel::B => 2,
            }
        }

        pub fn iterator() -> Iter<'static, Channel> {
            static CHANNELS: [Channel; 3] = [Channel::R, Channel::G, Channel::B];
            CHANNELS.iter()
        }
    }
}

mod pixel {

    use std::ops::{Index, IndexMut};

    use super::channel::Channel;

    #[derive(Clone)]
    pub struct Pixel<T> {
        pub r: T,
        pub g: T,
        pub b: T,
    }

    impl<T> Index<&Channel> for Pixel<T> {
        type Output = T;

        fn index(&self, c: &Channel) -> &T {
            match *c {
                Channel::R => &self.r,
                Channel::G => &self.g,
                Channel::B => &self.b,
            }
        }
    }

    impl<T> IndexMut<&Channel> for Pixel<T> {
        fn index_mut(&mut self, c: &Channel) -> &mut T {
            match *c {
                Channel::R => &mut self.r,
                Channel::G => &mut self.g,
                Channel::B => &mut self.b,
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn index_pixel() {
            let pixel = Pixel { r: 14, g: 9, b: 10 };
            assert_eq!(pixel[&Channel::R], 14);
            assert_eq!(pixel[&Channel::G], 9);
            assert_eq!(pixel[&Channel::B], 10);
        }

    }

}

mod dct {

    use super::channel::Channel;
    use super::pixel::Pixel;
    use crate::zigzag::Zigzag;

    /*Discrete Cosine Transformation Implementation*/

    fn dct(
        block: &[Vec<Pixel<u8>>],
        dct_block: &mut Vec<Vec<Pixel<f64>>>,
        u: usize,
        v: usize,
        c: &Channel,
        x_start: usize,
        x_length: usize,
        y_start: usize,
        y_length: usize,
    ) {
        let c_uv: f64 = match (u, v) {
            (0, 0) => 1.0 / 2.0,
            (0, _) => ((1.0 / 2.0) as f64).sqrt(),
            (_, 0) => ((1.0 / 2.0) as f64).sqrt(),
            _ => 1.0,
        };

        let mut f_uv = 0.0;
        for y in 0..y_length {
            for x in 0..x_length {
                let f_xy = block[y + y_start][x + x_start][c];

                f_uv += (f64::from(f_xy))
                    * ((((2 * x + 1) * u) as f64) * std::f64::consts::PI / 16.0).cos()
                    * ((((2 * y + 1) * v) as f64) * std::f64::consts::PI / 16.0).cos();
            }
        }

        f_uv *= 1.0 / 4.0 * c_uv;

        dct_block[v + y_start][u + x_start][c] = f_uv;
    }

    fn idct(
        block: &mut Vec<Vec<Pixel<u8>>>,
        dct_block: &[Vec<Pixel<f64>>],
        x: usize,
        y: usize,
        c: &Channel,
        x_start: usize,
        x_length: usize,
        y_start: usize,
        y_length: usize,
    ) {
        let mut f_xy = 0.0;
        for v in 0..y_length {
            for u in 0..x_length {
                let f_uv = dct_block[v + y_start][u + x_start][c];
                let c_uv: f64 = match (u, v) {
                    (0, 0) => 1.0 / 2.0,
                    (0, _) => ((1.0 / 2.0) as f64).sqrt(),
                    (_, 0) => ((1.0 / 2.0) as f64).sqrt(),
                    _ => 1.0,
                };
                f_xy += (c_uv as f64)
                    * f_uv
                    * ((((2 * x + 1) * u) as f64) * std::f64::consts::PI / 16.0).cos()
                    * ((((2 * y + 1) * v) as f64) * std::f64::consts::PI / 16.0).cos();
            }
        }

        f_xy *= 1.0 / 4.0;

        f_xy = f_xy.round();

        block[y + y_start][x + x_start][c] = if f_xy < 0.0 {
            0
        } else if f_xy > 255.0 {
            255
        } else {
            f_xy as u8
        };
    }

    pub fn dct_encode_block(
        block: &[Vec<Pixel<u8>>],
        frequencies: &mut Vec<Vec<Pixel<f64>>>,
        number: usize,
        block_size: usize,
        x_start: usize,
        x_length: usize,
        y_start: usize,
        y_length: usize,
    ) {
        let mut zigzag = Zigzag::new(block_size * block_size, block_size, block_size);
        for (u, v) in zigzag.by_ref().take(number) {
            for c in Channel::iterator() {
                dct(
                    block,
                    frequencies,
                    u,
                    v,
                    c,
                    x_start,
                    x_length,
                    y_start,
                    y_length,
                );
            }
        }
        for (u, v) in zigzag.by_ref() {
            for c in Channel::iterator() {
                frequencies[v + y_start][u + x_start][c] = 0.0;
            }
        }
    }

    pub fn dct_decode_block(
        block: &mut Vec<Vec<Pixel<u8>>>,
        frequencies: &[Vec<Pixel<f64>>],
        x_start: usize,
        x_length: usize,
        y_start: usize,
        y_length: usize,
    ) {
        for y in 0..y_length {
            for x in 0..x_length {
                for c in Channel::iterator() {
                    idct(
                        block,
                        frequencies,
                        x,
                        y,
                        c,
                        x_start,
                        x_length,
                        y_start,
                        y_length,
                    );
                }
            }
        }
    }
}

mod dwt {

    use super::channel::Channel;
    use super::pixel::Pixel;
    use crate::zigzag::Zigzag;

    /*Discrete Wavelet Transformation Implementation*/

    fn dwt(
        dwt_block: &mut Vec<Vec<Pixel<f64>>>,
        c: &Channel,
        by_row: bool,
        x_start: usize,
        x_length: usize,
        y_start: usize,
        y_length: usize,
    ) {
        let mut tmp_block: Vec<Vec<f64>> = vec![vec![0.0; x_length]; y_length];

        if by_row {
            for y in 0..y_length {
                for x in 0..x_length / 2 {
                    let pixel_1 = dwt_block[y + y_start][2 * x + x_start][c];
                    let pixel_2 = dwt_block[y + y_start][2 * x + 1 + x_start][c];

                    tmp_block[y][x] = (pixel_1 + pixel_2) as f64 / 2.0;
                    tmp_block[y][x + x_length / 2] = (pixel_1 - pixel_2) as f64 / 2.0;
                }
            }
        } else {
            for x in 0..x_length {
                for y in 0..y_length / 2 {
                    let pixel_1 = dwt_block[2 * y + y_start][x + x_start][c];
                    let pixel_2 = dwt_block[2 * y + 1 + y_start][x + x_start][c];

                    tmp_block[y][x] = (pixel_1 + pixel_2) as f64 / 2.0;
                    tmp_block[y + y_length / 2][x] = (pixel_1 - pixel_2) as f64 / 2.0;
                }
            }
        }

        for y in 0..y_length {
            for x in 0..x_length {
                dwt_block[y + y_start][x + x_start][c] = tmp_block[y][x];
            }
        }
    }

    fn idwt(
        dwt_block: &mut Vec<Vec<Pixel<f64>>>,
        c: &Channel,
        by_row: bool,
        x_start: usize,
        x_length: usize,
        y_start: usize,
        y_length: usize,
    ) {
        let mut tmp_block: Vec<Vec<f64>> = vec![vec![0.0; x_length]; y_length];

        if by_row {
            for y in 0..y_length {
                for x in 0..x_length / 2 {
                    let average = dwt_block[y + y_start][x + x_start][c];
                    let difference = dwt_block[y + y_start][x + x_length / 2 + x_start][c];

                    tmp_block[y][2 * x] = average + difference;
                    tmp_block[y][2 * x + 1] = average - difference;
                }
            }
        } else {
            for x in 0..x_length {
                for y in 0..y_length / 2 {
                    let average = dwt_block[y + y_start][x + x_start][c];
                    let difference = dwt_block[y + y_length / 2 + y_start][x + x_start][c];

                    tmp_block[2 * y][x] = average + difference;
                    tmp_block[2 * y + 1][x] = average - difference;
                }
            }
        }

        for y in 0..y_length {
            for x in 0..x_length {
                dwt_block[y + y_start][x + x_start][c] = tmp_block[y][x];
            }
        }
    }

    pub fn dwt_encode_block(
        dwt_block: &mut Vec<Vec<Pixel<f64>>>,
        number: usize,
        c: &Channel,
        x_start: usize,
        x_length: usize,
        y_start: usize,
        y_length: usize,
    ) {
        let mut x_dwt_blocksize = x_length;
        let mut y_dwt_blocksize = y_length;
        let mut by_row = true;

        while x_dwt_blocksize > 1 || y_dwt_blocksize > 1 {
            dwt(
                dwt_block,
                c,
                by_row,
                x_start,
                x_dwt_blocksize,
                y_start,
                y_dwt_blocksize,
            );
            if !by_row {
                x_dwt_blocksize /= 2;
                y_dwt_blocksize /= 2;
            }

            by_row = !by_row;
        }

        /*use coefficient in zigzag order*/

        if number == 0 {
            for y in y_start..y_start + y_length {
                for x in x_start..x_start + x_length {
                    dwt_block[y][x][c] = 0.0;
                }
            }
            return;
        }

        // TODO: change the axis using x_start and x_length
        // calculate largest square
        let mut counter: u32 = 0;
        while number > (4_usize.pow(counter)) {
            counter += 1;
        }
        if counter > 0 {
            counter -= 1;
        }

        let side_length: usize = 2_usize.pow(counter);
        let area = side_length * side_length;

        // zero
        if number > 3 * area {
            let skip_number = number - 3 * area;
            let zigzag = Zigzag::new(area, side_length, side_length);
            for (x, y) in zigzag.skip(skip_number) {
                dwt_block[y + side_length][x + side_length][c] = 0.0;
            }
        } else {
            // let skip_number_right = (number - area) / 2;
            // let skip_number_down = number - area - skip_number_right;

            let skip_number_down = std::cmp::min(number - area, area);
            let skip_number_right = number - area - skip_number_down;

            {
                let zigzag = Zigzag::new(area, side_length, side_length);
                for (x, y) in zigzag.skip(skip_number_right) {
                    dwt_block[y][x + side_length][c] = 0.0;
                }
            }
            {
                let zigzag = Zigzag::new(area, side_length, side_length);
                for (x, y) in zigzag.skip(skip_number_down) {
                    dwt_block[y + side_length][x][c] = 0.0;
                }
            }

            for y in side_length..2 * side_length {
                for x in side_length..2 * side_length {
                    dwt_block[y][x][c] = 0.0;
                }
            }
        }

        // zero rest value

        for y in 2 * side_length..y_length {
            for x in 0..x_length {
                dwt_block[y][x][c] = 0.0;
            }
        }

        for y in 0..y_length {
            for x in 2 * side_length..x_length {
                dwt_block[y][x][c] = 0.0;
            }
        }
    }

    pub fn dwt_decode_block(
        dwt_block: &mut Vec<Vec<Pixel<f64>>>,
        c: &Channel,
        x_start: usize,
        x_length: usize,
        y_start: usize,
        y_length: usize,
    ) {
        let mut x_dwt_blocksize = 1;
        let mut y_dwt_blocksize = 1;
        let mut by_row = false;

        while x_dwt_blocksize < x_length || y_dwt_blocksize < y_length || by_row {
            if !by_row {
                x_dwt_blocksize *= 2;
                y_dwt_blocksize *= 2;
            }
            idwt(
                dwt_block,
                c,
                by_row,
                x_start,
                x_dwt_blocksize,
                y_start,
                y_dwt_blocksize,
            );

            by_row = !by_row;
        }
    }
}

#[derive(Clone)]
pub struct Image {
    width: usize,
    height: usize,
    coefficient: usize,
    blocksize: usize,
    pixels: Option<Vec<Vec<Pixel<u8>>>>,
    frequencies: Option<Vec<Vec<Pixel<f64>>>>,
}

impl Image {
    pub fn get_coefficient(&self) -> usize {
        self.coefficient
    }

    pub fn set_coefficient(&mut self, coefficent: usize) -> &mut Self {
        self.coefficient = coefficent;
        self
    }
}

impl Image {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            coefficient: 0,
            blocksize: 0,
            pixels: None,
            frequencies: None,
        }
    }

    pub fn new_from_rgb(
        width: usize,
        height: usize,
        coefficient: usize,
        blocksize: usize,
        data: &[u8],
    ) -> Result<Self, std::io::Error> {
        let mut pixels: Vec<Vec<Pixel<u8>>> = vec![vec![Pixel { r: 0, g: 0, b: 0 }; width]; height];
        for c in Channel::iterator() {
            for y in 0..height {
                for x in 0..width {
                    let index = c.to_number() * width * height + y * width + x;
                    pixels[y][x][c] = data[index];
                }
            }
        }

        Ok(Self {
            width,
            height,
            coefficient,
            blocksize,
            pixels: Some(pixels),
            frequencies: None,
        })
    }

    pub fn to_1d_vec(&self) -> Vec<u8> {
        let mut ret: Vec<u8> = vec![0; self.width * self.height * 3];
        let stride = self.width * 3;
        let pixels = self
            .pixels
            .as_ref()
            .expect("to_1d_vec, image pixel could not be empty");
        for c in Channel::iterator() {
            for y in 0..self.height {
                for x in 0..self.width {
                    ret[y * stride + 3 * x + c.to_number()] = pixels[y][x][c];
                }
            }
        }
        ret
    }
}

impl Image {
    pub fn dct_encode(&mut self) {
        let frequencies: Vec<Vec<Pixel<f64>>> = vec![
            vec![
                Pixel {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0
                };
                self.width
            ];
            self.height
        ];
        self.frequencies = Some(frequencies);

        let number = (self.coefficient as f64
            / ((self.width / self.blocksize) * (self.height / self.blocksize)) as f64)
            .round() as usize;

        for y_block in 0..self.height / self.blocksize {
            for x_block in 0..self.width / self.blocksize {
                dct::dct_encode_block(
                    self.pixels
                        .as_ref()
                        .expect("encode, image pixel could not be empty"),
                    self.frequencies.as_mut().unwrap(),
                    number,
                    self.blocksize,
                    x_block * self.blocksize,
                    self.blocksize,
                    y_block * self.blocksize,
                    self.blocksize,
                );
            }
        }
    }

    pub fn dct_decode(&mut self) {
        let pixels: Vec<Vec<Pixel<u8>>> =
            vec![vec![Pixel { r: 0, g: 0, b: 0 }; self.width]; self.height];
        self.pixels = Some(pixels);

        for y_block in 0..self.height / self.blocksize {
            for x_block in 0..self.width / self.blocksize {
                dct::dct_decode_block(
                    self.pixels.as_mut().unwrap(),
                    self.frequencies
                        .as_ref()
                        .expect("decode, image frequencies could not be empty"),
                    x_block * self.blocksize,
                    self.blocksize,
                    y_block * self.blocksize,
                    self.blocksize,
                );
            }
        }
    }

    pub fn dwt_encode(&mut self) {
        let frequencies = self
            .pixels
            .as_ref()
            .unwrap()
            .iter()
            .map(|row| {
                row.iter()
                    .map(|p| Pixel {
                        r: f64::from(p.r),
                        g: f64::from(p.g),
                        b: f64::from(p.b),
                    })
                    .collect()
            })
            .collect();
        self.frequencies = Some(frequencies);

        for c in Channel::iterator() {
            dwt::dwt_encode_block(
                self.frequencies.as_mut().unwrap(),
                self.coefficient,
                c,
                0,
                self.width,
                0,
                self.height,
            );
        }
    }

    pub fn dwt_decode(&mut self) {
        for c in Channel::iterator() {
            dwt::dwt_decode_block(
                self.frequencies
                    .as_mut()
                    .expect("decode, image frequencies could not be empty"),
                c,
                0,
                self.width,
                0,
                self.height,
            );
        }

        let pixels = self
            .frequencies
            .as_ref()
            .unwrap()
            .iter()
            .map(|row| {
                row.iter()
                    .map(|p| {
                        let r = if p.r < 0.0 {
                            0
                        } else if p.r > 255.0 {
                            255
                        } else {
                            p.r.round() as u8
                        };

                        let g = if p.g < 0.0 {
                            0
                        } else if p.g > 255.0 {
                            255
                        } else {
                            p.g.round() as u8
                        };

                        let b = if p.b < 0.0 {
                            0
                        } else if p.b > 255.0 {
                            255
                        } else {
                            p.b.round() as u8
                        };

                        Pixel { r, g, b }
                    })
                    .collect()
            })
            .collect();
        self.pixels = Some(pixels);
    }
}
