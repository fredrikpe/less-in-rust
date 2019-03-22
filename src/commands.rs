
use std::fs::File;

use grep::matcher::Match;

use file_buffer::BiBufReader;
use input::Input;
use searcher;
use util;
use valid_reader::ValidReader;

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
    JumpNextMatch,
    JumpPrevMatch,

    Search(String),

    Quit,
    NoOp,
}

pub struct State {
    pub reader: BiBufReader<ValidReader<File>>,
    pub quit: bool,
    command_line: CommandLine,
    input_file: String,
    pub matches: Vec<(u64, Match)>,
}

impl State {
    pub fn new(input_file: &str) -> State {
        let file = match File::open(input_file) {
            Ok(f) => f,
            // TODO: Fails on file not found
            Err(_) => panic!("panic in state constructor"),
        };

        State {
            reader: BiBufReader::new(ValidReader::new(file)),
            quit: false,
            command_line: CommandLine::new(),
            input_file: input_file.to_string(),
            matches: Vec::new(),
        }
    }

    pub fn update(&mut self, input: &Input) -> Result<(), std::io::Error> {
        let command = self.command_line.parse_input(input);

        match command {
            Command::UpOneLine => self.reader.up_n_lines(1)?,
            Command::DownOneLine => self.reader.down_n_lines(1)?,
            Command::DownHalfScreen => {
                self.reader.down_n_lines(util::screen_height_half())?
            }
            Command::UpHalfScreen => {
                self.reader.up_n_lines(util::screen_height_half())?
            }
            Command::DownOneScreen => {
                self.reader.down_n_lines(util::screen_height())?
            }
            Command::UpOneScreen => {
                self.reader.up_n_lines(util::screen_height())?
            }
            Command::JumpBeginning => self.reader.jump_percentage(0)?,
            Command::JumpEnd => self.reader.jump_end()?,
            Command::JumpPercent(p) => self.reader.jump_percentage(p)?,
            Command::JumpNextMatch => self.try_jump_next_match()?,
            Command::JumpPrevMatch => self.try_jump_prev_match()?,
            Command::Search(pattern) => {
                match searcher::search(
                    &mut self.matches,
                    &self.input_file,
                    &pattern,
                ) {
                    Err(_) => (),
                    Ok(_) => (),
                };
            }
            Command::Quit => {
                self.quit = true;
            }
            _ => (),
        }

        Ok(())
    }

    fn try_jump_next_match(&mut self) -> Result<(), std::io::Error> {
        let cur_offset = self.reader.current_offset()?;
        return match self
            .matches
            .iter()
            .find(|(offset, _)| *offset > cur_offset)
        {
            Some((offset, _)) => self.reader.jump_offset(*offset),
            None => Ok(()),
        };
    }

    fn try_jump_prev_match(&mut self) -> Result<(), std::io::Error> {
        let cur_offset = self.reader.current_offset()?;
        return match self
            .matches
            .iter()
            .rev()
            .find(|(offset, _)| *offset < cur_offset)
        {
            Some((offset, _)) => self.reader.jump_offset(*offset),
            None => Ok(()),
        };
    }

    pub fn page(&mut self) -> (u64, Vec<u8>) {
        return match self.reader.page() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{}", e);
                (1, Vec::new())
            }
        };
    }

    pub fn command_line_text(&self) -> String {
        return self.command_line.text();
    }
}

enum Mode {
    Normal,
    Search,
}

struct CommandLine {
    mode: Mode,
    buffer: String,
}

impl CommandLine {
    pub fn new() -> CommandLine {
        CommandLine {
            mode: Mode::Normal,
            buffer: String::new(),
        }
    }

    pub fn parse_input(&mut self, input: &Input) -> Command {
        return match self.mode {
            Mode::Normal => self.normal_parse(input),
            Mode::Search => self.search_parse(input),
        };
    }

    fn normal_parse(&mut self, input: &Input) -> Command {
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
            Input::Char('p') => Command::JumpPercent(self.number()),

            Input::Char('n') => Command::JumpNextMatch,
            Input::Char('N') => Command::JumpPrevMatch,

            Input::Char('/') => {
                self.mode = Mode::Search;
                Command::NoOp
            }

            Input::Num(c) => {
                self.buffer.push(*c);
                Command::NoOp
            }

            Input::Ctrl(_) => Command::NoOp,
            _ => Command::NoOp,
        };
        if command != Command::NoOp {
            self.buffer.clear();
        }

        command
    }

    fn search_parse(&mut self, input: &Input) -> Command {
        let command = match input {
            Input::Ctrl('c') => {
                self.mode = Mode::Normal;
                Command::NoOp
            }

            Input::Char('\n') => {
                eprintln!("enter pressed");
                let pattern = self.buffer.clone();
                self.buffer.clear();
                self.mode = Mode::Normal;
                Command::Search(pattern)
            }

            Input::Backspace => {
                eprintln!("backspace");
                self.buffer.pop();
                Command::NoOp
            }

            Input::Char(c) => {
                self.buffer.push(*c);
                Command::NoOp
            }

            Input::Num(c) => {
                self.buffer.push(*c);
                Command::NoOp
            }

            _ => Command::NoOp,
        };
        if command != Command::NoOp {}

        command
    }

    pub fn text(&self) -> String {
        return match self.mode {
            Mode::Normal => match self.number() {
                0 => format!(":"),
                n => format!(":{}", n),
            },
            Mode::Search => format!("/{}", self.buffer),
        };
    }

    fn number(&self) -> u64 {
        let mut tot = 0;
        for (i, c) in self.buffer.chars().enumerate() {
            tot += c.to_digit(10).unwrap() as u64
                * 10u64.pow((self.buffer.len() - i - 1) as u32);
        }
        tot
    }
}
