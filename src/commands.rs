
use std::io::{Read, Seek};

use input::Input;
use util;
use file_buffer::BiBufReader;


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
}

impl<R: Read + Seek> State<R> {
    pub fn new(reader: BiBufReader<R>) -> State<R> {
        State {
            reader: reader,
            quit: false,
            mode: Mode::Normal,
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
            Command::Quit => { self.quit = true; },
            Command::NoOp => (),
            _ => (),
        }

        Ok(())
    }

    pub fn command(&mut self, input: &Input) -> Command {
        return match self.mode {
            Mode::Normal => self.normal_command(input),
            Mode::Search => self.normal_command(input),
        }
    }

    fn normal_command(&mut self, input: &Input) -> Command {
        return match input {
            Input::Char('q') => Command::Quit,

            Input::Ctrl('d') => Command::DownHalfScreen,
            Input::Ctrl('u') => Command::UpHalfScreen,

            Input::Ctrl('f') => Command::DownOneScreen,
            Input::Ctrl('b') => Command::UpOneScreen,

            Input::Char('j') => Command::DownOneLine,
            Input::Char('k') => Command::UpOneLine,
            Input::Char('g') => Command::JumpBeginning,
            Input::Char('G') => Command::JumpEnd,
            _ => Command::NoOp,
        }
    }


    pub fn page(&mut self)-> Vec<u8> {
        return self.reader.page().unwrap()
    }
}

pub struct NormalMode {
    command: Command,
}

impl NormalMode {
    
}

