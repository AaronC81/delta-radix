use std::time::Duration;

use async_trait::async_trait;
use delta_radix_hal::{Display, Keypad, Key, Time, Hal};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[wasm_bindgen]
extern "C" {
    fn radix_display_init();
    fn radix_display_clear();
    fn radix_display_print_char(c: char);
    fn radix_display_set_position(x: u8, y: u8);
    fn radix_display_get_position() -> Box<[u8]>;
}
pub struct WebDisplay;
impl Display for WebDisplay {
    fn init(&mut self) { radix_display_init() }
    fn clear(&mut self) { radix_display_clear() }
    fn print_char(&mut self, c: char) { radix_display_print_char(c) }
    fn set_position(&mut self, x: u8, y: u8) { radix_display_set_position(x, y) }
    fn get_position(&mut self) -> (u8, u8) { 
        let pos = radix_display_get_position();
        (pos[0], pos[1])
    }
}

#[wasm_bindgen]
extern "C" {
    async fn radix_keypad_wait_key() -> JsValue;
}
pub struct WebKeypad;
#[async_trait(?Send)]
impl Keypad for WebKeypad {
    async fn wait_key(&mut self) -> Key {
        let value = radix_keypad_wait_key().await;
        match value.as_string().expect("non-string returned from `radix_keypad_wait_key`").as_str() {
            x if x.len() == 1 && x.chars().next().unwrap().is_digit(16) => {
                Key::Digit(char::to_digit(x.chars().next().unwrap(), 16).unwrap() as u8)
            },

            "shift" => Key::Shift,
            "menu" => Key::Menu,
            "var" => todo!(),
            "left" => Key::Left,
            "right" => Key::Right,

            "add" => Key::Add,
            "subtract" => Key::Subtract,
            "multiply" => Key::Multiply,
            "divide" => Key::Divide,
            "delete" => Key::Delete,

            "format" => Key::FormatSelect,
            "hex" => Key::HexBase,
            "bin" => Key::BinaryBase,
            "exe" => Key::Exe,

            _ => panic!("unknown keypad key"),
        }
    }
}

#[wasm_bindgen]
extern "C" {
    async fn radix_time_sleep(ms: usize);
}
pub struct WebTime;
#[async_trait(?Send)]
impl Time for WebTime {
    async fn sleep(&mut self, dur: Duration) {
        radix_time_sleep(dur.as_millis() as usize).await;
    }
}

pub struct WebHal {
    display: WebDisplay,
    keypad: WebKeypad,
    time: WebTime,
}

impl WebHal {
    pub fn new() -> Self {
        Self {
            display: WebDisplay,
            keypad: WebKeypad,
            time: WebTime,
        }
    }
}

#[async_trait(?Send)]
impl Hal for WebHal {
    type D = WebDisplay;
    type K = WebKeypad;
    type T = WebTime;

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
        let (display, _, time) = self.common_mut();
        display.clear();
        display.set_position(3, 1);
        display.print_string("No bootloader");
        time.sleep(Duration::from_secs(2)).await;
    }   
}
