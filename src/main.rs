#![feature(align_offset)]
#![feature(read_initializer)]

extern crate clap;
extern crate grep;
extern crate libc;
extern crate termion;
extern crate unicode_segmentation;

use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;

use std::fs::File;
use std::io::{stdin, stdout};
use std::path::Path;
use std::result::Result;

use clap::{App, Arg};

mod args;
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
    let matches = App::new("My Super Program")
        .version("1.0")
        .author("Kevin K. <kbknapp@gmail.com>")
        .about("Does awesome things")
        .arg(
            Arg::with_name("input_files")
                //.short("if")
                //.long("input_file")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true)
                .required(true)
                .multiple(true),
        )
        .get_matches();

    // Swap stdin and TTY
    if !termion::is_tty(&stdin()) {
        // https://stackoverflow.com/a/29694013
        unsafe {
            use std::os::unix::io::*;

            let tty = File::open("/dev/tty").unwrap();

            let stdin_fd = libc::dup(0);

            let ret = File::from_raw_fd(stdin_fd);

            libc::dup2(tty.as_raw_fd(), 0);

            ::std::mem::forget(tty);

            Some(ret);
        }
    }

    let input_file = matches.value_of("input_files").unwrap();
    eprintln!("Input paths {:?}", Path::new(input_file));

    if let Err(_) = run(input_file) {
        std::process::exit(1);
    }
}

fn run(input_file: &str) -> Result<(), ()> {
    let mut printer = printer::Printer::new(stdout().into_raw_mode().unwrap());

    let mut state = commands::State::new(input_file);

    loop {
        let _ = printer.render(
            &mut state.page(),
            &state.matches,
            state.command_line_text().clone(),
        );

        // Blocks waiting for input.
        // Screen is not redrawn until input is registered.
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
