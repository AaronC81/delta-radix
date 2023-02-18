use std::{collections::VecDeque, time::Duration, panic::catch_unwind};

use async_trait::async_trait;
use delta_radix_hal::{Key, Display, Keypad, Time, Hal};
use delta_radix_os::main;
use futures::executor::block_on;
use panic_message::panic_message;

pub struct TestDisplay {
    lines: [String; 4],
    cursor: (u8, u8),
}

impl TestDisplay {
    pub fn new() -> Self {
        TestDisplay {
            lines: [
                " ".repeat(20),
                " ".repeat(20),
                " ".repeat(20),
                " ".repeat(20),
            ],
            cursor: (0, 0)
        }
    }
}

impl Display for TestDisplay {
    fn init(&mut self) {
        self.clear();
    }

    fn clear(&mut self) {
        *self = TestDisplay::new();
    }

    fn print_char(&mut self, c: char) {
        self.lines[self.cursor.1 as usize].replace_range(
            (self.cursor.0 as usize)..(self.cursor.0 as usize + 1),
            &c.to_string()
        );
        self.cursor.0 += 1;
    }

    fn set_position(&mut self, x: u8, y: u8) {
        self.cursor = (x, y)
    }
    fn get_position(&mut self) -> (u8, u8) {
        self.cursor
    }
}

pub struct TestKeypad {
    key_queue: VecDeque<Key>,
}
#[async_trait(?Send)]
impl Keypad for TestKeypad {
    async fn wait_key(&mut self) -> Key {
        self.key_queue.pop_front().expect("no more keys")
    }
}

pub struct TestTime;
#[async_trait(?Send)]
impl Time for TestTime {
    async fn sleep(&mut self, _: Duration) {}
}

pub struct TestHal {
    display: TestDisplay,
    keypad: TestKeypad,
    time: TestTime,
}

impl TestHal {
    pub fn new(keys: &[Key]) -> Self {
        Self {
            display: TestDisplay::new(),
            keypad: TestKeypad { key_queue: keys.iter().copied().collect() },
            time: TestTime,
        }
    }

    pub fn display_contents(&self) -> String {
        self.display.lines.join("\n")
    }

    pub fn display_line(&self, index: usize) -> String {
        self.display.lines[index].clone()
    }

    pub fn result(&self) -> String {
        self.display_line(3).trim().to_string()
    }

    pub fn expression(&self) -> String {
        self.display_line(2).trim().to_string()
    }
}

#[async_trait(?Send)]
impl Hal for TestHal {
    type D = TestDisplay;
    type K = TestKeypad;
    type T = TestTime;

    fn display(&self) -> &Self::D { &self.display }
    fn display_mut(&mut self) -> &mut Self::D { &mut self.display }

    fn keypad(&self) -> &Self::K { &self.keypad }
    fn keypad_mut(&mut self) -> &mut Self::K { &mut self.keypad }

    fn time(&self) -> &Self::T { &self.time }
    fn time_mut(&mut self) -> &mut Self::T { &mut self.time }

    fn common_mut(&mut self) -> (&mut Self::D, &mut Self::K, &mut Self::T) {
        (&mut self.display, &mut self.keypad, &mut self.time)
    }

    async fn enter_bootloader(&mut self) {
        panic!("test entered bootloader")
    }
}

pub fn run_os(keys: &[Key]) -> TestHal {
    let mut hal = TestHal::new(
        &keys.iter().chain(&[Key::DebugTerminate]).copied().collect::<Vec<_>>()[..]
    );
    let hal_ptr = &mut hal as *mut TestHal;
    
    match catch_unwind(|| block_on(main(unsafe { hal_ptr.as_mut().unwrap() }))) {
        // This is what we expect from pressing the DebugTerminate key!
        Err(e) if panic_message(&e) == "debug terminate" => (),

        Ok(()) => panic!("OS returned early"),
        Err(e) => panic!("panic within OS: {:?}", panic_message(&e))
    }

    hal
}
