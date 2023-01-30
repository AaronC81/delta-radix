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
}
