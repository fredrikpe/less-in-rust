use std::fs::File;
use std::io::{Error, ErrorKind};

use grep::matcher::Match;
use grep::regex::RegexMatcher;

use file_buffer;
use file_buffer::BiBufReader;
use input::Input;
use searcher;
use standard::StandardSink;
use util;

#[derive(Debug, PartialEq)]
pub enum Command {
    UpOneLine,
    UpHalfScreen,
    UpOneScreen,

    DownOneLine,
    DownHalfScreen,
    DownOneScreen,

    JumpBeginning,
    JumpEnd,
    JumpPercent(u64),

    Search(String),

    Quit,
    NoOp,
}

pub enum Mode {
    Normal,
    Search,
}

pub struct State {
    pub reader: BiBufReader<File>,
    pub quit: bool,
    mode: Mode,
    buf: Vec<u32>,
    pattern: String,
    input_file: String,
    pub matches: Vec<(u64, Match)>,
}

impl State {
    pub fn new(input_file: &str) -> State {
        let file = match File::open(input_file) {
            Ok(f) => f,
            Err(_) => panic!("panic in state constructor"),
        };

        State {
            reader: file_buffer::BiBufReader::new(file),
            quit: false,
            mode: Mode::Normal,
            buf: Vec::new(),
            pattern: String::new(),
            input_file: input_file.to_string(),
            matches: Vec::new(),
        }
    }

    pub fn update(&mut self, input: &Input) -> Result<(), std::io::Error> {
        let command = self.command(input);

        match command {
            Command::UpOneLine => self.reader.up_n_lines(1)?,
            Command::DownOneLine => self.reader.down_n_lines(1)?,
            Command::DownHalfScreen => self.reader.down_n_lines(util::screen_height_half())?,
            Command::UpHalfScreen => self.reader.up_n_lines(util::screen_height_half())?,
            Command::DownOneScreen => self.reader.down_n_lines(util::screen_height())?,
            Command::UpOneScreen => self.reader.up_n_lines(util::screen_height())?,
            Command::JumpBeginning => self.reader.jump_percentage(0)?,
            Command::JumpEnd => self.reader.jump_end()?,
            Command::JumpPercent(p) => self.reader.jump_percentage(p)?,
            Command::Search(_) => {
                self.matches.clear();
                self.do_search()?
            }
            Command::Quit => {
                self.quit = true;
            }
            _ => (),
        }

        Ok(())
    }

    pub fn command(&mut self, input: &Input) -> Command {
        return match self.mode {
            Mode::Normal => self.normal_command(input),
            Mode::Search => self.search_command(input),
        };
    }

    fn normal_command(&mut self, input: &Input) -> Command {
        let command = match input {
            Input::Char('q') => Command::Quit,

            Input::Ctrl('d') => Command::DownHalfScreen,
            Input::Ctrl('u') => Command::UpHalfScreen,

            Input::Ctrl('f') => Command::DownOneScreen,
            Input::Ctrl('b') => Command::UpOneScreen,

            Input::Char('j') => Command::DownOneLine,
            Input::Char('k') => Command::UpOneLine,

            Input::Char('g') => Command::JumpBeginning,
            Input::Char('G') => Command::JumpEnd,
            Input::Char('p') => Command::JumpPercent(self.total_number()),

            Input::Char('/') => {
                self.mode = Mode::Search;
                Command::NoOp
            }

            Input::Num(_c, n) => {
                self.add_number(*n);
                Command::NoOp
            }

            Input::Ctrl(_) => Command::NoOp,
            _ => Command::NoOp,
        };
        if command != Command::NoOp {
            self.buf.clear();
        }

        command
    }

    fn search_command(&mut self, input: &Input) -> Command {
        let command = match input {
            Input::Ctrl('c') => {
                self.mode = Mode::Normal;
                Command::NoOp
            }

            Input::Char('\n') => {
                eprintln!("enter pressed");
                Command::Search(self.pattern.clone())
            }

            Input::Backspace => {
                eprintln!("backspace");
                self.pattern.pop();
                Command::NoOp
            }

            Input::Char(c) => {
                self.pattern.push(*c);
                Command::NoOp
            }

            Input::Num(c, _n) => {
                self.pattern.push(*c);
                Command::NoOp
            }

            _ => Command::NoOp,
        };
        if command != Command::NoOp {}

        command
    }

    fn do_search(&mut self) -> Result<(), std::io::Error> {
        let mut sink = StandardSink {
            // TODO: Fails on invalid patterns
            matcher: RegexMatcher::new(&self.pattern[..]).unwrap(),
            matches: &mut self.matches,
            match_count: 0,
        };

        let path = std::path::Path::new(&self.input_file);
        let ret = match searcher::search_path(&mut sink, path) {
            Err(_e) => Err(Error::new(ErrorKind::Other, "Error in searcher.")),
            _ => Ok(()),
        };
        self.pattern.clear();
        self.mode = Mode::Normal;

        ret
    }

    fn add_number(&mut self, n: u32) {
        self.buf.push(n);
    }

    fn total_number(&self) -> u64 {
        let mut tot = 0;
        for (i, n) in self.buf.iter().enumerate() {
            tot += *n as u64 * 10u64.pow((self.buf.len() - i - 1) as u32);
        }
        tot
    }

    pub fn page(&mut self) -> (u64, Vec<u8>) {
        return self.reader.page().unwrap();
    }

    pub fn command_line_text(&self) -> String {
        return match self.mode {
            Mode::Normal => match self.total_number() {
                0 => format!(":"),
                n => format!(":{}", n),
            },
            Mode::Search => format!("/{}", self.pattern),
        };
    }
}

pub struct NormalMode {
    command: Command,
}

impl NormalMode {}
