use clap::{App as ClapApp, Arg, ArgMatches};
use std::fs::File;
use std::io::stdin;

use file_buffer::{InputReader, StdinCursor};

pub struct App {
    pub matches: ArgMatches<'static>,
}

impl App {
    pub fn new() -> App {
        App {
            matches: clap_app().get_matches(),
        }
    }

    pub fn input_source(&self) -> (InputReader, File) {
        let stdin = stdin();
        let input_file = self.matches.value_of("FILE");

        // Swap stdin and TTY
        let input = if !termion::is_tty(&stdin) {
            // https://stackoverflow.com/a/29694013
            unsafe {
                use std::os::unix::io::*;
                let tty = File::open("/dev/tty").unwrap();
                let stdin_fd = libc::dup(0);
                let file = File::from_raw_fd(stdin_fd);
                let file_copy = File::from_raw_fd(stdin_fd);
                libc::dup2(tty.as_raw_fd(), 0);
                ::std::mem::forget(tty);

                (InputReader::Stdin(StdinCursor::new(file)), file_copy)
            }
        } else {
            match input_file {
                Some(filename) => (
                    InputReader::File(File::open(filename).unwrap()),
                    File::open(filename).unwrap(),
                ),
                // Must have a filename as input.
                None => {
                    eprintln!("Expected 'rager <input>' or input over stdin.");
                    ::std::process::exit(1);
                }
            }
        };
        input
    }
}

fn clap_app() -> ClapApp<'static, 'static> {
    ClapApp::new("less2")
        .version("0.0.1")
        .about("About")
        .long_about("Long about")
        .arg(
            Arg::with_name("FILE")
                .help("File to view.")
                .long_help("File to view.")
                .multiple(false)
                .empty_values(false),
        )
        .help_message("Print this help message.")
        .version_message("Show version information.")
}
