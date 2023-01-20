use alloc::{vec::Vec, vec};
use delta_radix_hal::{Hal, Display, Keypad, Key};

mod glyph;
use self::glyph::Glyph;

pub struct CalculatorApplication<'h, H: Hal> {
    hal: &'h mut H,
    glyphs: Vec<Glyph>,
    cursor_pos: usize,
}

impl<'h, H: Hal> CalculatorApplication<'h, H> {
    pub fn new(hal: &'h mut H) -> Self {
        Self {
            hal,
            glyphs: vec![],
            cursor_pos: 0,
        }
    }

    pub async fn main(&mut self) {
        self.draw_full();

        loop {
            let key = self.hal.keypad_mut().wait_key().await;
            self.process_input_and_redraw(key);
        }
    }

    fn draw_full(&mut self) {
        self.hal.display_mut().clear();
        self.draw_header();
        self.draw_expression();
    }
    
    fn draw_header(&mut self) {
        let disp = self.hal.display_mut();
        disp.set_position(0, 0);
        disp.print_string("U32 ============ 50%");
    }

    fn draw_expression(&mut self) {
        let disp = self.hal.display_mut();
        disp.set_position(0, 2);

        let mut chars_written = 0;
        for glyph in &self.glyphs {
            disp.print_char(glyph.to_char());
            chars_written += 1;
        }

        for _ in chars_written..20 {
            disp.print_char(' ');
        }
    }

    fn process_input_and_redraw(&mut self, key: Key) {
        match key {
            Key::Digit(d) => self.insert_and_redraw(Glyph::Digit(d)),
            Key::HexBase => self.insert_and_redraw(Glyph::HexBase),
            Key::BinaryBase => self.insert_and_redraw(Glyph::BinaryBase),

            Key::Shift => (),
            Key::Menu => (),
        }
    }

    fn insert_and_redraw(&mut self, glyph: Glyph) {
        self.glyphs.insert(self.cursor_pos, glyph);
        self.cursor_pos += 1;
        self.draw_expression();
    }
}
