pub trait Display {
    fn init(&mut self);
    fn clear(&mut self);

    fn print_char(&mut self, c: char);

    fn set_position(&mut self, x: u8, y: u8);
    fn get_position(&self) -> (u8, u8);

    fn print_string(&mut self, s: &str) {
        for c in s.chars() {
            self.print_char(c)
        }
    }
}
