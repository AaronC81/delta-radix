use core::time::Duration;

use alloc::boxed::Box;
use async_trait::async_trait;
use cortex_m::delay::Delay;
use delta_radix_hal::Key;
use hd44780_driver::{HD44780, bus::FourBitBus, Cursor, CursorBlink};
use rp_pico::hal::gpio::{PinId, Pin, Output, PushPull, bank0::{Gpio11, Gpio10, Gpio9, Gpio8, Gpio7, Gpio6}};

type LcdRs = Gpio11;
type LcdEn = Gpio10;
type LcdD4 = Gpio9;
type LcdD5 = Gpio8;
type LcdD6 = Gpio7;
type LcdD7 = Gpio6;

// TODO: Write own LCD driver... position setting appears to be weird on this one, and I need
// custom characters which this doesn't support
pub struct LcdDisplay<'d> {
    pub delay: &'d mut Delay,
    pub lcd: HD44780<
        FourBitBus<
            Pin<LcdRs, Output<PushPull>>,
            Pin<LcdEn, Output<PushPull>>,
            Pin<LcdD4, Output<PushPull>>,
            Pin<LcdD5, Output<PushPull>>,
            Pin<LcdD6, Output<PushPull>>,
            Pin<LcdD7, Output<PushPull>>,
        >
    >
}

impl<'d> delta_radix_hal::Display for LcdDisplay<'d> {
    fn init(&mut self) {
        self.clear();
        self.lcd.set_cursor_visibility(Cursor::Invisible, self.delay);
        self.lcd.set_cursor_blink(CursorBlink::Off, self.delay);
        self.set_position(0, 0);
    }

    fn clear(&mut self) {
        self.lcd.clear(self.delay);
    }

    fn print_char(&mut self, c: char) {
        self.lcd.write_char(c, self.delay);
    }

    fn print_string(&mut self, s: &str) {
        self.lcd.write_str(s, self.delay);
    }

    fn set_position(&mut self, x: u8, y: u8) {
        self.lcd.set_cursor_pos(y * 40 + x, self.delay);
    }

    fn get_position(&mut self) -> (u8, u8) {
        // TODO
        (0, 0)
    }
}

pub struct ButtonMatrix;

#[async_trait(?Send)]
impl delta_radix_hal::Keypad for ButtonMatrix {
    async fn wait_key(&self) -> Key {
        // TODO
        loop {}
    }
}

pub struct DelayTime<'d> {
    pub delay: &'d mut Delay,
}

#[async_trait(?Send)]
impl<'d> delta_radix_hal::Time for DelayTime<'d> {
    async fn sleep(&mut self, dur: Duration) {
        self.delay.delay_ms(dur.as_millis() as u32)
    }
}

pub struct PicoHal<'d> {
    pub display: LcdDisplay<'d>,
    pub keypad: ButtonMatrix,
    pub time: DelayTime<'d>,
}

impl<'d> delta_radix_hal::Hal for PicoHal<'d> {
    type D = LcdDisplay<'d>;
    type K = ButtonMatrix;
    type T = DelayTime<'d>;

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
