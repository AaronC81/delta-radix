use alloc::{vec::Vec, vec};
use delta_radix_hal::{Hal, Display, Keypad, Key};

mod glyph;
use self::{glyph::{Glyph, Base}, eval::{EvaluationResult, Configuration, DataType}, parse::{Parser, Node, ParserError}};

pub mod num;

mod eval;
mod parse;

pub struct CalculatorApplication<'h, H: Hal> {
    hal: &'h mut H,
    glyphs: Vec<Glyph>,
    cursor_pos: usize,
    eval_config: Configuration,
    eval_result: Option<EvaluationResult>,
}

impl<'h, H: Hal> CalculatorApplication<'h, H> {
    pub fn new(hal: &'h mut H) -> Self {
        Self {
            hal,
            glyphs: vec![],
            cursor_pos: 0,
            eval_config: Configuration {
                data_type: DataType {
                    bits: 32,
                    signed: false,
                }
            },
            eval_result: None,
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
        // Try to parse and get warning spans
        let (parser, _) = self.parse();
        let warning_indices = parser.constant_overflow_spans.iter()
            .map(|s| s.indices().collect::<Vec<_>>())
            .flatten().collect::<Vec<_>>();
        
        let disp = self.hal.display_mut();

        // Draw expression
        disp.set_position(0, 2);
        let mut chars_written = 0;
        for glyph in &self.glyphs {
            disp.print_char(glyph.to_char());
            chars_written += 1;
        }
        for _ in chars_written..20 {
            disp.print_char(' ');
        }

        // Draw cursor
        disp.set_position(0, 1);
        for i in 0..20 {
            if i + 1 == self.cursor_pos {
                disp.print_char('\\')
            } else if i == self.cursor_pos {
                disp.print_char('/')
            } else {
                if warning_indices.contains(&i) {
                    disp.print_char('!')
                } else {
                    disp.print_char(' ')
                }
            }
        }
    }

    fn process_input_and_redraw(&mut self, key: Key) {
        match key {
            Key::Digit(d) => self.insert_and_redraw(Glyph::Digit(d)),
            Key::HexBase => self.insert_and_redraw(Glyph::Base(Base::Hexadecimal)),
            Key::BinaryBase => self.insert_and_redraw(Glyph::Base(Base::Binary)),

            Key::Add => self.insert_and_redraw(Glyph::Add),
            Key::Subtract => self.insert_and_redraw(Glyph::Subtract),
            Key::Multiply => self.insert_and_redraw(Glyph::Multiply),
            Key::Divide => self.insert_and_redraw(Glyph::Divide),

            Key::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    self.draw_expression();
                }
            },
            Key::Right => {
                if self.cursor_pos < self.glyphs.len() {
                    self.cursor_pos += 1;
                    self.draw_expression();
                }
            }
            Key::Delete => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    self.glyphs.remove(self.cursor_pos);
                    self.draw_expression();
                }
            },

            Key::Shift => (),
            Key::Menu => (),
        }
    }

    fn insert_and_redraw(&mut self, glyph: Glyph) {
        self.glyphs.insert(self.cursor_pos, glyph);
        self.cursor_pos += 1;
        self.draw_expression();
    }

    fn parse(&self) -> (Parser, Result<Node, ParserError>) {
        let mut parser = Parser::new(&self.glyphs, self.eval_config);
        let result = parser.parse();
        (parser, result)
    }
}
