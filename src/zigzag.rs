pub struct Zigzag {
    i: usize,
    j: usize,
    cur: usize,         // index by 0
    max: usize,         // index by 1
    blocksize_x: usize, // width
    blocksize_y: usize, // height
}

impl Zigzag {
    pub fn new(max: usize, blocksize_x: usize, blocksize_y: usize) -> Self {
        Self {
            i: 1,
            j: 1,
            cur: 0,
            max,
            blocksize_x,
            blocksize_y,
        }
    }
}

impl Iterator for Zigzag {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<(usize, usize)> {
        if self.cur < self.max && self.cur < self.blocksize_x * self.blocksize_y {
            // coordination (j, i) width, height (x, y)
            let old = (self.j - 1, self.i - 1);

            if (self.i + self.j) % 2 == 0 {
                // right-up
                if self.j < self.blocksize_x {
                    self.j += 1;
                } else {
                    self.i += 2;
                }
                if self.i > 1 {
                    self.i -= 1;
                }
            } else {
                // down-left
                if self.i < self.blocksize_y {
                    self.i += 1;
                } else {
                    self.j += 2;
                }
                if self.j > 1 {
                    self.j -= 1;
                }
            }

            self.cur += 1;

            Some(old)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zigzag() {
        let mut z = Zigzag::new(16, 4, 4).into_iter();

        for i in z.by_ref().take(25) {
            println!("{:?}", i);
        }
    }

    #[test]
    fn test_zigzag_rec() {
        let blocksize_x = 5;
        let blocksize_y = 3;
        let mut z = Zigzag::new(blocksize_x * blocksize_y, blocksize_x, blocksize_y).into_iter();

        for i in z.by_ref().take(blocksize_x * blocksize_y) {
            println!("{:?}", i);
        }
    }

}
