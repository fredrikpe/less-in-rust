#![feature(read_initializer)]

extern crate clap;
extern crate grep;
extern crate termion;
extern crate unicode_segmentation;

use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;

use std::fs::File;
use std::io::stdout;
use std::result::Result;

use clap::{App, Arg};

mod commands;
mod file_buffer;
mod input;
mod printer;
mod searcher;
mod string_util;
mod util;

fn main() {
    let matches = App::new("My Super Program")
        .version("1.0")
        .author("Kevin K. <kbknapp@gmail.com>")
        .about("Does awesome things")
        .arg(
            Arg::with_name("input_file")
                .short("if")
                .long("input_file")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let input_file = matches.value_of("input_file").unwrap();

    if let Err(e) = run(input_file) {
        std::process::exit(1);
    }
}

fn run(input_file: &str) -> Result<(), ()> {
    let mut printer = printer::Printer {
        out: AlternateScreen::from(stdout().into_raw_mode().unwrap()),
    };

    let mut state = commands::State::new(input_file);

    let mut input_event = input::Input::NoOp;

    loop {
        let _ = printer.render(&state.page(), state.command_line_text());

        let input = input::parse_input();

        if let Err(e) = state.update(&input) {
            eprintln!("Error in state.update: {}", e);
        }

        if state.quit {
            break;
        }
    }

    Ok(())
}
