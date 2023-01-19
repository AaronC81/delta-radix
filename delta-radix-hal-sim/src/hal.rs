use std::{io::{stdout, Write, Stdout, Stdin, stdin}, cell::RefCell, process::exit};

use async_trait::async_trait;
use delta_radix_hal::{Display, Keypad, Key, Hal};
use termion::{raw::{IntoRawMode, RawTerminal}, input::{TermRead, Keys}};

pub struct SimDisplay {
    x: u8,
    y: u8,
    stdout: RawTerminal<Stdout>,
}

impl SimDisplay {
    fn new() -> Self {
        let stdout = stdout().into_raw_mode().unwrap();
        Self { stdout, x: 0, y: 0 }
    }
}

impl Display for SimDisplay {
    fn init(&mut self) {
        write!(self.stdout, "{}", termion::clear::All).unwrap();
        self.set_position(0, 0);
        self.stdout.flush().unwrap();
    }

    fn print_char(&mut self, c: char) {
        write!(self.stdout, "{}", c).unwrap();
        self.stdout.flush().unwrap();
    }

    fn set_position(&mut self, x: u8, y: u8) {
        self.x = x;
        self.y = y;
        write!(self.stdout, "{}", termion::cursor::Goto(x as u16 + 1, y as u16 + 1)).unwrap();
    }

    fn get_position(&self) -> (u8, u8) {
        (self.x, self.y)
    }
}

pub struct SimKeypad {
    keys: RefCell<Keys<Stdin>>,
}

impl SimKeypad {
    fn new() -> Self {
        let keys = RefCell::new(stdin().keys());
        Self { keys }
    }
}

#[async_trait(?Send)]
impl Keypad for SimKeypad {
    async fn wait_key(&self) -> Key {
        loop {
            match self.keys.borrow_mut().next().unwrap().unwrap() {
                termion::event::Key::Char(' ') => return Key::Menu,
                termion::event::Key::Char('s') => return Key::Shift,
                termion::event::Key::Char('q') => panic!("exit"),
                termion::event::Key::Char(c) if c.is_digit(10)
                    => return Key::Digit(c.to_digit(10).unwrap() as u8),

                _ => (),
            };
        }
    }
}

pub struct SimHal {
    display: SimDisplay,
    keypad: SimKeypad,
}

impl SimHal {
    pub fn new() -> Self {
        Self {
            display: SimDisplay::new(),
            keypad: SimKeypad::new(),
        }
    }
}

impl Hal for SimHal {
    type D = SimDisplay;
    type K = SimKeypad;

    fn display(&self) -> &Self::D { &self.display }
    fn display_mut(&mut self) -> &mut Self::D { &mut self.display }

    fn keypad(&self) -> &Self::K { &self.keypad }
    fn keypad_mut(&mut self) -> &mut Self::K { &mut self.keypad }
}
