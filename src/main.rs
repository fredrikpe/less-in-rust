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
mod string_util;
mod screen;

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

    let _metadata = match file.metadata() {
        Ok(m) => m,
        Err(_) => return (),
    };

    let mut state = State { quit: false };

    let mut bi_reader = file_buffer::BiBufReader::new(std::io::BufReader::new(file));

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
            _ => {
                eprintln!("NoOp");
            }
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
        let _ = bi_reader.move_to_command(&input_event);

        let _ = printer.print_screen(bi_reader);

        input_event = input::get_input();
        printer.flush();
        state.update(&input_event);

        if state.quit {
            break;
        }
    }

    Ok(())
}
