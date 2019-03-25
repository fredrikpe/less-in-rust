#![feature(align_offset)]
#![feature(read_initializer)]

extern crate clap;
extern crate grep;
extern crate libc;
extern crate termion;
extern crate unicode_segmentation;

use termion::raw::IntoRawMode;

use std::io::{stdin, stdout};
use std::path::Path;
use std::result::Result;

mod app;
mod controller;
mod error;
mod reader;
mod input;
mod printer;
mod searcher;
mod standard;
mod string_util;
mod utf8_validation;
mod util;

fn main() {
    let app = app::App::new();

    let input_reader = app.input_reader();

    if let Err(_) = run(input_reader) {
        std::process::exit(1);
    }
}

fn run(input_file: reader::InputReader) -> Result<(), ()> {
    let mut printer = printer::Printer::new(stdout().into_raw_mode().unwrap());

    let mut controller = controller::Controller::new(input_file);

    loop {
        let _ = printer.render(
            &mut controller.page(),
            &controller.matches,
            controller.command_line_text().clone(),
        );

        // Blocks, waiting for input.
        // Screen is not redrawn until input is registered.
        let input = input::get_input();

        if let Err(e) = controller.update(&input) {
            eprintln!("Error in controller.update: {}", e);
        }

        if controller.quit {
            break;
        }
    }

    Ok(())
}
