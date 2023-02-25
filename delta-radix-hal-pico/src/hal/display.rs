use cortex_m::delay::Delay;
use delta_radix_hal::{DisplaySpecialCharacter, Glyph};
use hd44780_driver::{bus::FourBitBus, HD44780, Cursor, CursorBlink};
use rp_pico::hal::gpio::{bank0::{Gpio11, Gpio10, Gpio9, Gpio8, Gpio7, Gpio6, Gpio5}, Output, Pin, PushPull};

type LcdRs = Gpio11;
type LcdEn = Gpio10;
type LcdD4 = Gpio9;
type LcdD5 = Gpio8;
type LcdD6 = Gpio7;
type LcdD7 = Gpio6;

type LcdBacklight = Gpio5;

pub struct LcdDisplay<'d> {
    pub delay: &'d mut Delay,
    pub backlight: Pin<LcdBacklight, Output<PushPull>>,
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

pub struct CustomChar {
    pub index: u8,
    pub data: [u8; 8],
}

impl CustomChar {
    pub const fn new(index: u8, data: [u8; 8]) -> Self {
        Self { index, data }
    }

    pub fn register<'d>(&self, display: &mut LcdDisplay<'d>) {
        display.lcd.set_custom_char(self.index, self.data, display.delay).unwrap();
    }
}

mod chars {
    use super::CustomChar;

    pub const CURSOR_LEFT: CustomChar = CustomChar::new(0, [
        0b00000000,
        0b00000000,
        0b00000000,
        0b00000000,
        0b00000000,
        0b00000100,
        0b00000010,
        0b00000001,
    ]);

    pub const CURSOR_RIGHT: CustomChar = CustomChar::new(1, [
        0b00000000,
        0b00000000,
        0b00000000,
        0b00000000,
        0b00000000,
        0b00000100,
        0b00001000,
        0b00010000,
    ]);

    pub const WARNING: CustomChar = CustomChar::new(2, [
        0b00000000,
        0b00000000,
        0b00000000,
        0b00000000,
        0b00000000,
        0b00000000,
        0b00010101,
        0b00000000,
    ]);

    pub const CURSOR_LEFT_WITH_WARNING: CustomChar = CustomChar::new(3, [
        0b00000000,
        0b00000000,
        0b00000000,
        0b00000000,
        0b00000000,
        0b00000100,
        0b00010010,
        0b00000001,
    ]);

    pub const CURSOR_RIGHT_WITH_WARNING: CustomChar = CustomChar::new(4, [
        0b00000000,
        0b00000000,
        0b00000000,
        0b00000000,
        0b00000000,
        0b00000100,
        0b00001001,
        0b00010000,
    ]);

    pub const MULTIPLY: CustomChar = CustomChar::new(5, [
        0b00000000,
        0b00010001,
        0b00001010,
        0b00000100,
        0b00001010,
        0b00010001,
        0b00000000,
        0b00000000,
    ]);
}

impl<'d> LcdDisplay<'d> {
    /// The absolute cursor position at which each line starts.
    const CURSOR_LINE_OFFSETS: [u8; 4] = [
        // Yeah... I don't know why they're like this? I assume the 0x00 and 0x40 are for
        // compatibility with 2x16 displays, but uhh
        // These were just found by the Power of Guessing
        0x00,
        0x40,
        0x14,
        0x54,
    ];
}

impl<'d> delta_radix_hal::Display for LcdDisplay<'d> {
    fn init(&mut self) {
        chars::CURSOR_LEFT.register(self);
        chars::CURSOR_RIGHT.register(self);
        chars::WARNING.register(self);
        chars::CURSOR_LEFT_WITH_WARNING.register(self);
        chars::CURSOR_RIGHT_WITH_WARNING.register(self);
        chars::MULTIPLY.register(self);
        
        self.clear();

        self.lcd.set_cursor_visibility(Cursor::Invisible, self.delay).unwrap();
        self.lcd.set_cursor_blink(CursorBlink::Off, self.delay).unwrap();

        self.set_position(0, 0);
    }

    fn clear(&mut self) {
        self.lcd.clear(self.delay).unwrap();

        // This command seems to take a while - prevent garbage
        self.delay.delay_ms(10);
    }

    fn print_char(&mut self, c: char) {
        self.lcd.write_char(c, self.delay).unwrap();
    }

    fn print_string(&mut self, s: &str) {
        self.lcd.write_str(s, self.delay).unwrap();
    }

    fn set_position(&mut self, x: u8, y: u8) {
        self.lcd.set_cursor_pos(Self::CURSOR_LINE_OFFSETS[y as usize] + x, self.delay).unwrap();
    }

    fn get_position(&mut self) -> (u8, u8) {
        // TODO
        (0, 0)
    }

    fn print_special(&mut self, character: DisplaySpecialCharacter) {
        let byte = match character {
            DisplaySpecialCharacter::CursorLeft => chars::CURSOR_LEFT.index,
            DisplaySpecialCharacter::CursorRight => chars::CURSOR_RIGHT.index,
            DisplaySpecialCharacter::Warning => chars::WARNING.index,
            DisplaySpecialCharacter::CursorLeftWithWarning => chars::CURSOR_LEFT_WITH_WARNING.index,
            DisplaySpecialCharacter::CursorRightWithWarning => chars::CURSOR_RIGHT_WITH_WARNING.index,
        };
        self.lcd.write_byte(byte, self.delay).unwrap();
    }

    fn print_glyph(&mut self, glyph: Glyph) {
        self.print_char(
            match glyph {
                Glyph::Multiply => chars::MULTIPLY.index as char,
                // Not aligned with baseline of other operators, but it'll do!
                Glyph::Divide => 0b1111_1101 as char,
                _ => glyph.char(),
            }
        );
    }
}
