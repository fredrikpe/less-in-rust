use grep::matcher::Match;

use error::Result;
use input::{Command, CommandLine, UserInput};
use reader::{BiBufReader, FileSwitcher, InputReader, Search, ValidReader};
use util;

pub struct Controller {
    pub reader: BiBufReader<ValidReader<InputReader>>,
    pub quit: bool,
    command_line: CommandLine,
    pub matches: Vec<(u64, Match)>,
}

impl Controller {
    pub fn new(input_reader: InputReader, wrap: bool) -> Controller {
        Controller {
            reader: BiBufReader::new(ValidReader::new(input_reader), wrap),
            quit: false,
            command_line: CommandLine::new(),
            matches: Vec::new(),
        }
    }

    pub fn update(&mut self, input: &UserInput) -> Result<()> {
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

            Command::JumpNextMatch(is_forward) => {
                self.jump_next_match(is_forward)
            }

            Command::Search(pattern, is_forward) => {
                self.find_matches(&pattern);
                self.jump_next_match(is_forward)
            }

            Command::NextFile => self.next_file(),
            Command::Quit => {
                self.quit = true;
            }
            _ => (),
        }

        Ok(())
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

    pub fn is_wrap(&self) -> bool {
        self.reader.wrap
    }

    fn find_matches(&mut self, pattern: &str) {
        self.matches.clear();
        self.reader.search(&mut self.matches, pattern);
    }

    fn jump_next_match(&mut self, is_forward: bool) {
        let cur_offset = self.reader.current_offset();
        if is_forward {
            self.jump_forward_match(cur_offset)
        } else {
            self.jump_backward_match(cur_offset)
        };
    }

    fn jump_forward_match(&mut self, cur_offset: u64) {
        match self.matches.iter().find(|(offset, _)| *offset > cur_offset) {
            Some((offset, _)) => self.reader.jump_offset(*offset).unwrap(),
            None => (),
        }
    }

    fn jump_backward_match(&mut self, cur_offset: u64) {
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

    fn next_file(&mut self) {
        match self.reader.next_file() {
            Err(e) => eprintln!("{}", e),
            Ok(_) => {
                self.matches.clear();
                ()
            }
        }
    }
}
