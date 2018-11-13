extern crate clap;
extern crate termion;

use termion::screen::AlternateScreen;
use termion::raw::IntoRawMode;

use std::fs::File;
use std::result::Result;
use std::io::stdout;
use std::io::Seek;
use std::io::Read;

use clap::{Arg, App};


mod printer;
mod input;
mod file_buffer;

use input::LEvent;

fn main() {
    let matches = App::new("My Super Program")
                          .version("1.0")
                          .author("Kevin K. <kbknapp@gmail.com>")
                          .about("Does awesome things")
                          .arg(Arg::with_name("input_file")
                               .short("if")
                               .long("input_file")
                               .value_name("FILE")
                               .help("Sets a custom config file")
                               .takes_value(true)
                               .required(true))
                          .get_matches();


    let input_file = matches.value_of("input_file");
    let file = File::open(input_file.unwrap());

    let fileb = match file {
        Ok(t) => std::io::BufReader::new(t),
        Err(_) => return (),
    };

    let mut bi_reader = file_buffer::BiBufReader { reader: fileb, position: 0, size: 1024 };

    match run(&mut bi_reader) {
        Ok(()) => return (),
        Err(()) => return (),
    }
}

struct State {
    offset: i64,
    quit: bool,
}

impl State {
    pub fn update(&mut self, input_event: LEvent) {
        match input_event {
            LEvent::LineUp => self.offset = -1,
            LEvent::LineDown => {
                self.offset = 1
            },
            LEvent::Quit => self.quit = true,
            _ => self.offset = 0,
        }
    }
}

fn run<R: Read + Seek>(bi_reader: &mut file_buffer::BiBufReader<R>) -> Result<(), ()> {

    let mut printer = printer::Printer { 
        out: AlternateScreen::from(stdout().into_raw_mode().unwrap())
    };
    let mut state = State { offset: 0, quit: false };

    loop {
        let _ = printer.print_screen(state.offset, bi_reader);

        let input_event = input::get_input();
        printer.flush();
        state.update(input_event);

        if state.quit {
            break;
        }
    }

    Ok(())
}



