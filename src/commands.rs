use std::io::{Read, Seek};

use file_buffer::BiBufReader;
use input::Input;
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

pub struct State<R: Read + Seek> {
    reader: BiBufReader<R>,
    pub quit: bool,
    mode: Mode,
    buf: Vec<u32>,
}

impl<R: Read + Seek> State<R> {
    pub fn new(reader: BiBufReader<R>) -> State<R> {
        State {
            reader: reader,
            quit: false,
            mode: Mode::Normal,
            buf: Vec::new(),
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
            Command::Quit => {
                self.quit = true;
            }
            Command::NoOp => (),
            _ => (),
        }

        Ok(())
    }

    pub fn command(&mut self, input: &Input) -> Command {
        return match self.mode {
            Mode::Normal => self.normal_command(input),
            Mode::Search => self.normal_command(input),
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

            Input::Num(n) => {
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

    fn add_number(&mut self, n: u32) {
        self.buf.push(n);
    }

    fn total_number(&mut self) -> u64 {
        let mut tot = 0;
        for (i, n) in self.buf.iter().enumerate() {
            tot += *n as u64 * 10u64.pow((self.buf.len() - i - 1) as u32);
        }
        tot
    }

    pub fn page(&mut self) -> Vec<u8> {
        return self.reader.page().unwrap();
    }

    pub fn command_line_text(&mut self) -> String {
        return match self.total_number() {
            0 => format!(":"),
            n => format!(":{}", n),
        };
    }
}

pub struct NormalMode {
    command: Command,
}

impl NormalMode {}
