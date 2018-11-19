use termion::event::Event;
use termion::event::Key;
use termion::event::MouseButton;
use termion::event::MouseEvent;
use termion::input::TermRead;

use std::io::stdin;


#[derive(Debug)]
pub enum Input {
    Ctrl(char),
    Char(char),
    NoOp,
}


pub fn parse_input() -> Input {
    let stdin = stdin();
    for c in stdin.events() {
        return match c.unwrap() {
            Event::Key(Key::Char(c)) => Input::Char(c),

            Event::Key(Key::Ctrl(c)) => Input::Ctrl(c),

            //    Event::Mouse(MouseEvent::Press(MouseButton::WheelUp, _, _))
            
            _ =>  Input::NoOp,
        };
    }
     Input::NoOp
}

