use std::fs::File;
use std::io::{stdin, Stdin};

use grep::matcher::Match;

use reader::{BiBufReader, InputReader, Search, ValidReader};
use input::{Command, CommandLine, UserInput};
use searcher;
use util;

pub struct Controller {
    pub reader: BiBufReader<ValidReader<InputReader>>,
    pub quit: bool,
    command_line: CommandLine,
    file: File,
    pub matches: Vec<(u64, Match)>,
}

impl Controller {
    pub fn new(input_reader: InputReader, file: File) -> Controller {
        Controller {
            reader: BiBufReader::new(ValidReader::new(input_reader)),
            quit: false,
            command_line: CommandLine::new(),
            file: file,
            matches: Vec::new(),
        }
    }

    pub fn update(&mut self, input: &UserInput) -> Result<(), std::io::Error> {
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
            Command::JumpNextMatch => self.jump_next_match(),
            Command::JumpPrevMatch => self.jump_prev_match(),
            Command::Search(pattern) => {
                self.find_matches(&pattern);
                dbg!(self.matches.len());
                self.jump_next_match();
                self.jump_prev_match();
            }
            Command::Quit => {
                self.quit = true;
            }
            _ => (),
        }

        Ok(())
    }

    fn find_matches(&mut self, pattern: &str) {
        self.matches.clear();
        self.reader.search(&mut self.matches, pattern);
    }

    fn jump_next_match(&mut self) {
        let cur_offset = self.reader.current_offset().unwrap();

        match self.matches.iter().find(|(offset, _)| *offset > cur_offset) {
            Some((offset, _)) => self.reader.jump_offset(*offset).unwrap(),
            None => (),
        }
    }

    fn jump_prev_match(&mut self) {
        let cur_offset = self.reader.current_offset().unwrap();
        match self
            .matches
            .iter()
            .rev()
            .find(|(offset, _)| *offset < cur_offset)
        {
            Some((offset, _)) => self.reader.jump_offset(*offset).unwrap(),
            None => (),
        }
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
