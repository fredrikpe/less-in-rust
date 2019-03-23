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
mod commands;
mod error;
mod file_buffer;
mod input;
mod printer;
mod searcher;
mod standard;
mod string_util;
mod utf8_validation;
mod util;
mod valid_reader;

fn main() {
    let app = app::App::new();

    let input_file = app.files()[0].clone();
    dbg!(&input_file);

    if let Err(_) = run(input_file) {
        std::process::exit(1);
    }
}

fn run(input_file: app::InputSource) -> Result<(), ()> {
    let mut printer = printer::Printer::new(stdout().into_raw_mode().unwrap());

    let mut state = commands::State::new(input_file);

    loop {
        let _ = printer.render(
            &mut state.page(),
            &state.matches,
            state.command_line_text().clone(),
        );

        // Blocks, waiting for input.
        // Screen is not redrawn until input is registered.
        let input = input::get_input();

        if let Err(e) = state.update(&input) {
            eprintln!("Error in state.update: {}", e);
        }

        if state.quit {
            break;
        }
    }

    Ok(())
}
