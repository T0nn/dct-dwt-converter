extern crate gio;
extern crate gtk;

use gio::prelude::*;
use gtk::prelude::*;

mod config;
mod image;
mod zigzag;

use self::config::Config;
use self::image::Image;

fn build_ui(application: &gtk::Application) {
    let config = Config::new(std::env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        std::process::exit(1);
    });

    let content = std::fs::read(&config.path).expect("path illegal");
    // let name = config.path.file_stem().expect("extract the stem of file_name failed").to_string_lossy().into_owned();

    /*set up parameters*/
    let width = 512;
    let height = 512;
    let blocksize = 8;
    let coefficient = config.coefficient as usize;

    if config.coefficient == -1 {
        let mut image_rgb =
            Image::new_from_rgb(width, height, coefficient, blocksize, &content).unwrap();

        let mut image_dct_series: Vec<Image> = vec![Image::new(); 64];
        let mut image_dwt_series: Vec<Image> = vec![Image::new(); 64];

        let max_iteration = 64;
        for i in 0..max_iteration {
            println!("converting image iteration {} ..", i + 1);

            let base_coefficient = 4096;
            image_rgb.set_coefficient((i + 1) * base_coefficient);

            image_dct_series[i] = image_rgb.clone();
            image_dwt_series[i] = image_rgb.clone();

            /*encode using dct or dwt*/
            image_dct_series[i].dct_encode();
            image_dct_series[i].dct_decode();

            image_dwt_series[i].dwt_encode();
            image_dwt_series[i].dwt_decode();
        }

        let window = gtk::ApplicationWindow::new(application);

        window.set_title("CSCI576 Assginment2");
        window.set_border_width(10);
        window.set_position(gtk::WindowPosition::Center);
        window.set_default_size(1064, 630);

        let grid = gtk::Grid::new();
        let button = gtk::Button::new_with_label("Pasue");
        let button_clone = button.clone();
        let button_2 = gtk::Button::new_with_label("Restart");
        let label_0 = gtk::Label::new(image_dct_series[0].get_coefficient().to_string().as_str());
        let label_0_clone = label_0.clone();
        let label_0_clone_clone = label_0.clone();
        let label1 = gtk::Label::new("dct");
        let label2 = gtk::Label::new("dwt");

        let image_width: i32 = width as i32;
        let image_height: i32 = height as i32;

        // let mut image_data_1:Vec<u8> = vec![0; (image_width*image_width*3) as usize];
        let image_data_1 = image_dct_series[0].to_1d_vec();
        let pixbuf_1 = gdk_pixbuf::Pixbuf::new_from_mut_slice(
            image_data_1,
            gdk_pixbuf::Colorspace::Rgb,
            false,
            8,
            image_width,
            image_height,
            image_width * 3,
        );

        // let image_data_2:Vec<u8> = vec![255; (image_width*image_width*3) as usize];
        let image_data_2 = image_dwt_series[0].to_1d_vec();
        let pixbuf_2 = gdk_pixbuf::Pixbuf::new_from_mut_slice(
            image_data_2,
            gdk_pixbuf::Colorspace::Rgb,
            false,
            8,
            image_width,
            image_height,
            image_width * 3,
        );

        let image_1 = gtk::Image::new_from_pixbuf(&pixbuf_1);
        let image_1_clone = image_1.clone();
        let image_1_clone_clone = image_1.clone();
        let image_2 = gtk::Image::new_from_pixbuf(&pixbuf_2);
        let image_2_clone = image_2.clone();
        let image_2_clone_clone = image_2.clone();

        use std::sync::{Arc, Mutex};

        let image_dct = Arc::new(Mutex::new(image_dct_series));
        let image_dwt = Arc::new(Mutex::new(image_dwt_series));

        let image_dct_clone = Arc::clone(&image_dct);
        let image_dwt_clone = Arc::clone(&image_dwt);

        let pause = Arc::new(Mutex::new(false));
        let pause_clone = Arc::clone(&pause);
        let pause_clone_1 = Arc::clone(&pause);

        let interval = 800;
        // let mut flag = false;
        // let mut counter = 1;
        let counter = Arc::new(Mutex::new(1));
        let counter_clone = Arc::clone(&counter);
        let counter_clone_1 = Arc::clone(&counter);
        //  let mut barrier:usize = 0;
        gtk::timeout_add(interval, move || {
            // if barrier < 30 {
            //     barrier += 1;
            //     return gtk::Continue(true);
            // } else {
            //     barrier = 0;
            // }

            if *pause.lock().unwrap() {
                return gtk::Continue(true);
            }

            let mut counter_ptr = counter_clone.lock().unwrap();

            if *counter_ptr >= max_iteration {
                return gtk::Continue(true);
            }

            let image_dct_vec = image_dct.lock().unwrap();
            let image_dwt_vec = image_dwt.lock().unwrap();

            let pixbuf_1 = gdk_pixbuf::Pixbuf::new_from_mut_slice(
                (*image_dct_vec)[*counter_ptr].to_1d_vec(),
                gdk_pixbuf::Colorspace::Rgb,
                false,
                8,
                image_width,
                image_height,
                image_width * 3,
            );
            //            let pixbuf_2 = gdk_pixbuf::Pixbuf::new_from_mut_slice( (*image_dwt_vec)[*counterPtr].to_1d_vec(), gdk_pixbuf::Colorspace::Rgb, false, 8, image_width, image_height, image_width*3);

            let pixbuf_2 = gdk_pixbuf::Pixbuf::new_from_mut_slice(
                (*image_dwt_vec)[*counter_ptr].to_1d_vec(),
                gdk_pixbuf::Colorspace::Rgb,
                false,
                8,
                image_width,
                image_height,
                image_width * 3,
            );

            image_1_clone.set_from_pixbuf(&pixbuf_1);
            image_2_clone.set_from_pixbuf(&pixbuf_2);

            label_0_clone.set_label(
                (*image_dct_vec)[*counter_ptr]
                    .get_coefficient()
                    .to_string()
                    .as_str(),
            );

            *counter_ptr += 1;

            gtk::Continue(true)
        });

        button.connect_clicked(move |_| {
            let mut pasue_flag = pause_clone.lock().unwrap();
            *pasue_flag = !*pasue_flag;
            if *pasue_flag {
                button_clone.set_label("Resume");
            } else {
                button_clone.set_label("Pause");
            }
        });

        button_2.connect_clicked(move |_| {
            let mut counter_ptr = counter_clone_1.lock().unwrap();
            *counter_ptr = 0;

            *pause_clone_1.lock().unwrap() = false;

            if *counter_ptr >= max_iteration {
                return;
            }

            let image_dct_series = image_dct_clone.lock().unwrap();
            let image_dwt_series = image_dwt_clone.lock().unwrap();

            let pixbuf_1 = gdk_pixbuf::Pixbuf::new_from_mut_slice(
                (*image_dct_series)[*counter_ptr].to_1d_vec(),
                gdk_pixbuf::Colorspace::Rgb,
                false,
                8,
                image_width,
                image_height,
                image_width * 3,
            );
            let pixbuf_2 = gdk_pixbuf::Pixbuf::new_from_mut_slice(
                (*image_dwt_series)[*counter_ptr].to_1d_vec(),
                gdk_pixbuf::Colorspace::Rgb,
                false,
                8,
                image_width,
                image_height,
                image_width * 3,
            );

            image_1_clone_clone.set_from_pixbuf(&pixbuf_1);
            image_2_clone_clone.set_from_pixbuf(&pixbuf_2);

            label_0_clone_clone.set_label(
                (*image_dct_series)[*counter_ptr]
                    .get_coefficient()
                    .to_string()
                    .as_str(),
            );

            *counter_ptr += 1;
        });

        grid.attach(&label_0, 0, 0, 2, 1);
        grid.attach(&label1, 0, 1, 1, 1);
        grid.attach(&label2, 1, 1, 1, 1);
        grid.attach(&image_1, 0, 2, 1, 1);
        grid.attach(&image_2, 1, 2, 1, 1);

        grid.attach(&button, 0, 3, 1, 1);
        grid.attach(&button_2, 1, 3, 1, 1);

        grid.set_column_spacing(20);
        grid.set_row_spacing(20);

        window.add(&grid);

        window.show_all();
    } else {
        let image_rgb =
            Image::new_from_rgb(width, height, coefficient, blocksize, &content).unwrap();

        let mut image_dct = image_rgb.clone();
        let mut image_dwt = image_rgb.clone();

        /*encode using dct or dwt*/
        image_dct.dct_encode();
        image_dct.dct_decode();

        image_dwt.dwt_encode();
        image_dwt.dwt_decode();

        let window = gtk::ApplicationWindow::new(application);

        window.set_title("CSCI576 Assginment2");
        window.set_border_width(10);
        window.set_position(gtk::WindowPosition::Center);
        window.set_default_size(1064, 630);

        let grid = gtk::Grid::new();
        let label_0 = gtk::Label::new(config.coefficient.to_string().as_str());
        let label1 = gtk::Label::new("dct");
        let label2 = gtk::Label::new("dwt");

        let image_width: i32 = width as i32;
        let image_height: i32 = height as i32;

        let image_data_1 = image_dct.to_1d_vec();
        let pixbuf_1 = gdk_pixbuf::Pixbuf::new_from_mut_slice(
            image_data_1,
            gdk_pixbuf::Colorspace::Rgb,
            false,
            8,
            image_width,
            image_height,
            image_width * 3,
        );

        let image_data_2 = image_dwt.to_1d_vec();
        let pixbuf_2 = gdk_pixbuf::Pixbuf::new_from_mut_slice(
            image_data_2,
            gdk_pixbuf::Colorspace::Rgb,
            false,
            8,
            image_width,
            image_height,
            image_width * 3,
        );

        let image_1 = gtk::Image::new_from_pixbuf(&pixbuf_1);
        let image_2 = gtk::Image::new_from_pixbuf(&pixbuf_2);

        grid.attach(&label_0, 0, 0, 2, 1);
        grid.attach(&label1, 0, 1, 1, 1);
        grid.attach(&label2, 1, 1, 1, 1);
        grid.attach(&image_1, 0, 2, 1, 1);
        grid.attach(&image_2, 1, 2, 1, 1);

        grid.set_column_spacing(20);
        grid.set_row_spacing(20);

        window.add(&grid);

        window.show_all();
    }
}

fn main() {
    let application = gtk::Application::new("com.github.gtk-rs.examples.basic", Default::default())
        .expect("Initialization failed...");

    application.connect_activate(|app| {
        build_ui(app);
    });

    let empty: Vec<String> = Vec::new();
    application.run(&empty);
}
