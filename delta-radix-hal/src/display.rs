pub trait Display {
    fn init(&mut self);
    fn print_char(&mut self, c: char);

    fn set_position(&mut self, x: u8, y: u8);
    fn get_position(&self) -> (u8, u8);
}
