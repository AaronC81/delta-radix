#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Glyph {
    Digit(u8),

    Add,
    Subtract,
    Multiply,
    Divide,

    HexBase,
    BinaryBase,
    DecimalBase,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DisplaySpecialCharacter {
    CursorLeft,
    CursorRight,
    Warning,
    CursorLeftWithWarning,
    CursorRightWithWarning,
}

pub trait Display {
    fn init(&mut self);
    fn clear(&mut self);

    fn print_char(&mut self, c: char);

    fn set_position(&mut self, x: u8, y: u8);
    fn get_position(&mut self) -> (u8, u8);

    fn print_string(&mut self, s: &str) {
        for c in s.chars() {
            self.print_char(c)
        }
    }

    fn print_special(&mut self, character: DisplaySpecialCharacter) {
        self.print_char(
            match character {
                DisplaySpecialCharacter::CursorLeft => '\\',
                DisplaySpecialCharacter::CursorRight => '/',
                DisplaySpecialCharacter::Warning => '!',
                DisplaySpecialCharacter::CursorLeftWithWarning => '\\',
                DisplaySpecialCharacter::CursorRightWithWarning => '/',
            }
        )
    }

    fn print_glyph(&mut self, glyph: Glyph) {
        self.print_char(
            match glyph {
                Glyph::Digit(d) => char::from_digit(d as u32, 16).unwrap().to_uppercase().next().unwrap(),
    
                Glyph::Add => '+',
                Glyph::Subtract => '-',
                Glyph::Multiply => '×',
                Glyph::Divide => '÷',
    
                Glyph::HexBase => 'x',
                Glyph::BinaryBase => 'b',
                Glyph::DecimalBase => 'd',
            }
        )
    }
}
