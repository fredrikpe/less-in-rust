use clap::{App as ClapApp, Arg, ArgMatches};
use std::fs::File;
use std::io::stdin;

use reader::{InputReader, StdinCursor};

pub struct App {
    pub matches: ArgMatches<'static>,
}

impl App {
    pub fn new() -> App {
        App {
            matches: clap_app().get_matches(),
        }
    }

    pub fn input_reader(&self) -> InputReader {
        let files = self.matches
            .values_of("FILE")
            .map(|values| {
                values
                .map(|filename| {
                    File::open(filename).unwrap()
                })
                .collect()
            })
        .unwrap_or_else(|| Vec::new());

        return if files.len() > 0 {
            InputReader::Files(files)
        } else {
            let stdin = stdin();
            if termion::is_tty(&stdin) {
                eprintln!("Expected a file or input over stdin.");
            }
            self.stdin_reader()
        }
    }

    fn stdin_reader(&self) -> InputReader {
        unsafe {
            use std::os::unix::io::*;

            let tty = File::open("/dev/tty").unwrap();
            let stdin_fd = libc::dup(0);
            let file = File::from_raw_fd(stdin_fd);

            libc::dup2(tty.as_raw_fd(), 0);
            ::std::mem::forget(tty);

            InputReader::Stdin(StdinCursor::new(file))
        }
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
                .multiple(true)
                .empty_values(false),
        )
        .help_message("Print this help message.")
        .version_message("Show version information.")
}
