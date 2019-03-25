use termion::event::Event;
use termion::event::Key;
//use termion::event::MouseButton;
//use termion::event::MouseEvent;
use termion::input::TermRead;

use std::io::stdin;

#[derive(Debug)]
pub enum UserInput {
    Ctrl(char),
    Char(char),
    Num(char),
    Backspace,
    NoOp,
}

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

    NextFile,

    Quit,
    NoOp,
}

pub fn get_input() -> UserInput {
    let stdin = stdin();
    for c in stdin.events() {
        return match c.unwrap() {
            Event::Key(Key::Char(c)) => parse_char(c),

            Event::Key(Key::Ctrl(c)) => UserInput::Ctrl(c),

            Event::Key(Key::Backspace) => UserInput::Backspace,

            //    Event::Mouse(MouseEvent::Press(MouseButton::WheelUp, _, _))
            _ => UserInput::NoOp,
        };
    }
    UserInput::NoOp
}

fn parse_char(c: char) -> UserInput {
    return match c.to_digit(10) {
        Some(_) => UserInput::Num(c),
        None => UserInput::Char(c),
    };
}

enum Mode {
    Normal,
    Search,
}

pub struct CommandLine {
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

    pub fn parse_input(&mut self, input: &UserInput) -> Command {
        return match self.mode {
            Mode::Normal => self.normal_parse(input),
            Mode::Search => self.search_parse(input),
        };
    }

    fn normal_parse(&mut self, input: &UserInput) -> Command {
        use input::UserInput::*;

        let command = match input {
            Char('q') => Command::Quit,

            Ctrl('d') | Char('d') => Command::DownHalfScreen,
            Ctrl('u') | Char('u') => Command::UpHalfScreen,

            Ctrl('f') | Char('f') => Command::DownOneScreen,
            Ctrl('b') | Char('b') => Command::UpOneScreen,

            Char('j') => Command::DownOneLine,
            Char('k') => Command::UpOneLine,

            Char('g') => Command::JumpBeginning,
            Char('G') => Command::JumpEnd,
            Char('p') => Command::JumpPercent(self.number()),

            Char('n') => Command::JumpNextMatch,
            Char('N') => Command::JumpPrevMatch,

            Char('/') => {
                self.mode = Mode::Search;
                Command::NoOp
            }

            Num(c) => {
                self.buffer.push(*c);
                Command::NoOp
            }

            Char('>') => Command::NextFile,

            Ctrl(_) => Command::NoOp,
            _ => Command::NoOp,
        };
        if command != Command::NoOp {
            self.buffer.clear();
        }

        command
    }

    fn search_parse(&mut self, input: &UserInput) -> Command {
        let command = match input {
            UserInput::Ctrl('c') => {
                self.mode = Mode::Normal;
                Command::NoOp
            }

            UserInput::Char('\n') => {
                eprintln!("enter pressed");
                let pattern = self.buffer.clone();
                self.buffer.clear();
                self.mode = Mode::Normal;
                Command::Search(pattern)
            }

            UserInput::Backspace => {
                eprintln!("backspace");
                self.buffer.pop();
                Command::NoOp
            }

            UserInput::Char(c) => {
                self.buffer.push(*c);
                Command::NoOp
            }

            UserInput::Num(c) => {
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
