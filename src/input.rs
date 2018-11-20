use termion::event::Event;
use termion::event::Key;
//use termion::event::MouseButton;
//use termion::event::MouseEvent;
use termion::input::TermRead;

use std::io::stdin;

#[derive(Debug)]
pub enum Input {
    Ctrl(char),
    Char(char),
    Num(u32),
    NoOp,
}

pub fn parse_input() -> Input {
    let stdin = stdin();
    for c in stdin.events() {
        return match c.unwrap() {
            Event::Key(Key::Char(c)) => parse_char(c),

            Event::Key(Key::Ctrl(c)) => Input::Ctrl(c),

            //    Event::Mouse(MouseEvent::Press(MouseButton::WheelUp, _, _))
            _ => Input::NoOp,
        };
    }
    Input::NoOp
}

pub fn parse_char(c: char) -> Input {
    return match c.to_digit(10) {
        Some(n) => Input::Num(n),
        None => Input::Char(c),
    };
}
