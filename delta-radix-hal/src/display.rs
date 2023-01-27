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

    // These exist because the HD44780 character set shows \ as the Yen symbol, so we need to handle
    // it specially!
    fn print_cursor_left(&mut self) { self.print_char('\\') }
    fn print_cursor_right(&mut self) { self.print_char('/') }
}
