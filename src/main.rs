#![feature(read_initializer)]

extern crate clap;
extern crate termion;
extern crate unicode_segmentation;

use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;

use std::fs::File;
use std::io::stdout;
use std::io::Read;
use std::io::Seek;
use std::result::Result;

use clap::{App, Arg};

mod file_buffer;
mod input;
mod line_num_cache;
mod printer;
mod util;
mod string_util;

use input::LEvent;

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
        ).get_matches();

    let input_file = matches.value_of("input_file");
    let file = match File::open(input_file.unwrap()) {
        Ok(f) => f,
        Err(_) => return (),
    };

    let mut state = State { quit: false };

    let mut bi_reader = file_buffer::BiBufReader::new(file);

    match run(&mut state, &mut bi_reader) {
        Ok(()) => return (),
        Err(()) => return (),
    }
}

struct State {
    quit: bool,
}

impl State {
    pub fn update(&mut self, input_event: &LEvent) {
        match input_event {
            LEvent::UpOneLine => (),
            LEvent::DownOneLine => (),
            LEvent::Quit => self.quit = true,
            _ => {},
        }
    }
}

fn run<R: Read + Seek>(
    state: &mut State,
    bi_reader: &mut file_buffer::BiBufReader<R>,
) -> Result<(), ()> {
    let mut printer = printer::Printer {
        out: AlternateScreen::from(stdout().into_raw_mode().unwrap()),
    };

    let mut input_event = LEvent::NoOp;

    loop {
        if let Err(e) = update_reader(bi_reader, &input_event) {
            eprintln!("Error in update_reader: {}", e);
        }

        let _ = printer.print_screen(bi_reader);

        input_event = input::get_input();

        state.update(&input_event);

        if state.quit {
            break;
        }
    }

    Ok(())
}

fn update_reader<R: Read + Seek>(
    reader: &mut file_buffer::BiBufReader<R>,
    action: &LEvent,
) -> Result<(), std::io::Error> {
    match action {
        LEvent::UpOneLine => reader.up_n_lines(1)?,
        LEvent::DownOneLine => reader.down_n_lines(1)?,
        LEvent::DownHalfScreen => reader.down_n_lines(util::screen_height_half())?,
        LEvent::UpHalfScreen => reader.up_n_lines(util::screen_height_half())?,
        LEvent::DownOneScreen => reader.down_n_lines(util::screen_height())?,
        LEvent::UpOneScreen => reader.up_n_lines(util::screen_height())?,
        LEvent::JumpBeginning => reader.jump_percentage(0)?,
        LEvent::JumpEnd => reader.jump_end()?,
        LEvent::NoOp => (),
        _ => (),
    }

    Ok(())
}
