use cortex_m::delay::Delay;
use delta_radix_hal::DisplaySpecialCharacter;
use hd44780_driver::{bus::FourBitBus, HD44780, Cursor, CursorBlink};
use rp_pico::hal::gpio::{bank0::{Gpio11, Gpio10, Gpio9, Gpio8, Gpio7, Gpio6}, Output, Pin, PushPull};

type LcdRs = Gpio11;
type LcdEn = Gpio10;
type LcdD4 = Gpio9;
type LcdD5 = Gpio8;
type LcdD6 = Gpio7;
type LcdD7 = Gpio6;

// TODO: Need to support custom characters at some point

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

    const CUSTOM_CHAR_INDEX_CURSOR_LEFT: u8 = 0;
    const CUSTOM_CHAR_DATA_CURSOR_LEFT: [u8; 8] = [
        0b00000000,
        0b00000000,
        0b00000000,
        0b00000000,
        0b00000000,
        0b00000100,
        0b00000010,
        0b00000001,
    ];

    const CUSTOM_CHAR_INDEX_CURSOR_RIGHT: u8 = 1;
    const CUSTOM_CHAR_DATA_CURSOR_RIGHT: [u8; 8] = [
        0b00000000,
        0b00000000,
        0b00000000,
        0b00000000,
        0b00000000,
        0b00000100,
        0b00001000,
        0b00010000,
    ];

    const CUSTOM_CHAR_INDEX_WARNING: u8 = 2;
    const CUSTOM_CHAR_DATA_WARNING: [u8; 8] = [
        0b00000000,
        0b00000000,
        0b00000000,
        0b00000000,
        0b00000000,
        0b00000000,
        0b00010101,
        0b00000000,
    ];

    const CUSTOM_CHAR_INDEX_CURSOR_LEFT_WITH_WARNING: u8 = 3;
    const CUSTOM_CHAR_DATA_CURSOR_LEFT_WITH_WARNING: [u8; 8] = [
        0b00000000,
        0b00000000,
        0b00000000,
        0b00000000,
        0b00000000,
        0b00000100,
        0b00010010,
        0b00000001,
    ];

    const CUSTOM_CHAR_INDEX_CURSOR_RIGHT_WITH_WARNING: u8 = 4;
    const CUSTOM_CHAR_DATA_CURSOR_RIGHT_WITH_WARNING: [u8; 8] = [
        0b00000000,
        0b00000000,
        0b00000000,
        0b00000000,
        0b00000000,
        0b00000100,
        0b00001001,
        0b00010000,
    ];
}

impl<'d> delta_radix_hal::Display for LcdDisplay<'d> {
    fn init(&mut self) {
        self.lcd.set_custom_char(Self::CUSTOM_CHAR_INDEX_CURSOR_LEFT, Self::CUSTOM_CHAR_DATA_CURSOR_LEFT, self.delay).unwrap();
        self.lcd.set_custom_char(Self::CUSTOM_CHAR_INDEX_CURSOR_RIGHT, Self::CUSTOM_CHAR_DATA_CURSOR_RIGHT, self.delay).unwrap();
        self.lcd.set_custom_char(Self::CUSTOM_CHAR_INDEX_WARNING, Self::CUSTOM_CHAR_DATA_WARNING, self.delay).unwrap();
        self.lcd.set_custom_char(Self::CUSTOM_CHAR_INDEX_CURSOR_LEFT_WITH_WARNING, Self::CUSTOM_CHAR_DATA_CURSOR_LEFT_WITH_WARNING, self.delay).unwrap();
        self.lcd.set_custom_char(Self::CUSTOM_CHAR_INDEX_CURSOR_RIGHT_WITH_WARNING, Self::CUSTOM_CHAR_DATA_CURSOR_RIGHT_WITH_WARNING, self.delay).unwrap();
        
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
            DisplaySpecialCharacter::CursorLeft => Self::CUSTOM_CHAR_INDEX_CURSOR_LEFT,
            DisplaySpecialCharacter::CursorRight => Self::CUSTOM_CHAR_INDEX_CURSOR_RIGHT,
            DisplaySpecialCharacter::Warning => Self::CUSTOM_CHAR_INDEX_WARNING,
            DisplaySpecialCharacter::CursorLeftWithWarning => Self::CUSTOM_CHAR_INDEX_CURSOR_LEFT_WITH_WARNING,
            DisplaySpecialCharacter::CursorRightWithWarning => Self::CUSTOM_CHAR_INDEX_CURSOR_RIGHT_WITH_WARNING,
        };
        self.lcd.write_byte(byte, self.delay).unwrap();
    }
}
