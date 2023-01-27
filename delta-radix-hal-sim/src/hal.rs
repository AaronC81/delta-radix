use std::{io::{stdout, Write, Stdout, Stdin, stdin}, cell::RefCell, process::exit, time::Duration};

use async_trait::async_trait;
use delta_radix_hal::{Display, Keypad, Key, Hal, Time};
use termion::{raw::{IntoRawMode, RawTerminal}, input::{TermRead, Keys}};
use termion::event::Key as TermKey;

pub struct SimDisplay {
    x: u8,
    y: u8,
    stdout: RawTerminal<Stdout>,
}

impl SimDisplay {
    const ROWS: u8 = 4;
    const COLS: u8 = 20;

    fn new() -> Self {
        let stdout = stdout().into_raw_mode().unwrap();
        Self { stdout, x: 0, y: 0 }
    }

    fn draw_border(&mut self) {
        write!(self.stdout, "{}", termion::cursor::Goto(1, 1)).unwrap();
        write!(self.stdout, "┌{}┐\r\n", str::repeat("─", Self::COLS as usize)).unwrap();
        for _ in 0..Self::ROWS {
            write!(self.stdout, "│{}│\r\n", str::repeat(" ", Self::COLS as usize)).unwrap();
        }
        write!(self.stdout, "└{}┘", str::repeat("─", Self::COLS as usize)).unwrap();
    }
}

impl Display for SimDisplay {
    fn init(&mut self) {
        self.clear();
    }

    fn clear(&mut self) {
        write!(self.stdout, "{}", termion::clear::All).unwrap();
        self.draw_border();
        self.set_position(0, 0);
        self.stdout.flush().unwrap();
    }

    fn print_char(&mut self, c: char) {
        if self.x >= Self::COLS || self.y >= Self::ROWS {
            panic!("position ({}, {}) is out-of-range", self.x, self.y)
        }
        self.x += 1;

        write!(self.stdout, "{}", c).unwrap();
        self.stdout.flush().unwrap();
    }

    fn set_position(&mut self, x: u8, y: u8) {
        self.x = x;
        self.y = y;
        // +1 to make 1-indexed, +1 again to skip over the border
        write!(self.stdout, "{}", termion::cursor::Goto(x as u16 + 2, y as u16 + 2)).unwrap();
    }

    fn get_position(&mut self) -> (u8, u8) {
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

pub struct SimTime;

impl SimTime {
    fn new() -> Self { Self }
}

#[async_trait(?Send)]
impl Time for SimTime {
    async fn sleep(&mut self, dur: Duration) {
        tokio::time::sleep(dur).await
    }
}

#[async_trait(?Send)]
impl Keypad for SimKeypad {
    async fn wait_key(&mut self) -> Key {
        loop {
            match self.keys.borrow_mut().next().unwrap().unwrap() {                                
                TermKey::Char(c) if c.is_digit(10)
                    => return Key::Digit(c.to_digit(10).unwrap() as u8),
                TermKey::Char('x') => return Key::HexBase,
                TermKey::Char('b') => return Key::BinaryBase,

                TermKey::Char('+') => return Key::Add,
                TermKey::Char('-') => return Key::Subtract,
                TermKey::Char('*') => return Key::Multiply,
                TermKey::Char('/') => return Key::Divide,

                TermKey::Left => return Key::Left,
                TermKey::Right => return Key::Right,
                TermKey::Backspace => return Key::Delete,
                TermKey::Char('\n') => return Key::Exe,

                TermKey::Char(' ') => return Key::Menu,
                TermKey::Char('s') => return Key::Shift,
                TermKey::Char('q') => panic!("exit"),

                _ => (),
            };
        }
    }
}

pub struct SimHal {
    display: SimDisplay,
    keypad: SimKeypad,
    time: SimTime,
}

impl SimHal {
    pub fn new() -> Self {
        Self {
            display: SimDisplay::new(),
            keypad: SimKeypad::new(),
            time: SimTime::new(),
        }
    }
}

impl Hal for SimHal {
    type D = SimDisplay;
    type K = SimKeypad;
    type T = SimTime;

    fn display(&self) -> &Self::D { &self.display }
    fn display_mut(&mut self) -> &mut Self::D { &mut self.display }

    fn keypad(&self) -> &Self::K { &self.keypad }
    fn keypad_mut(&mut self) -> &mut Self::K { &mut self.keypad }

    fn time(&self) -> &Self::T { &self.time }
    fn time_mut(&mut self) -> &mut Self::T { &mut self.time }

    fn common_mut(&mut self) -> (&mut Self::D, &mut Self::K, &mut Self::T) {
        (&mut self.display, &mut self.keypad, &mut self.time)
    }
}
