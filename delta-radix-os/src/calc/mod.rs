use alloc::{vec::Vec, vec};
use delta_radix_hal::{Hal, Display, Keypad, Key};

mod glyph;
use self::{glyph::{Glyph, Base}, eval::{EvaluationResult, Configuration, DataType, evaluate}, parse::{Parser, Node, ParserError}};

pub mod num;

mod eval;
mod parse;

pub struct CalculatorApplication<'h, H: Hal> {
    hal: &'h mut H,
    glyphs: Vec<Glyph>,
    cursor_pos: usize,
    eval_config: Configuration,
    eval_result: Option<Result<EvaluationResult, ParserError>>,
    constant_overflows: bool,
}

impl<'h, H: Hal> CalculatorApplication<'h, H> {
    pub fn new(hal: &'h mut H) -> Self {
        Self {
            hal,
            glyphs: vec![],
            cursor_pos: 0,
            eval_config: Configuration {
                data_type: DataType {
                    bits: 16,
                    signed: false,
                }
            },
            eval_result: None,
            constant_overflows: false,
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
        self.draw_result();
    }
    
    fn draw_header(&mut self) {
        let disp = self.hal.display_mut();
        disp.set_position(0, 0);

        let name = self.eval_config.data_type.concise_name();
        disp.print_string(&name);
        disp.print_char(' ');

        let has_overflow = if let Some(Ok(r)) = &self.eval_result {
            r.overflow || self.constant_overflows
        } else {
            false
        };
        let overflow_marker = " OVER";

        let mut ptr = name.len() + 1;
        let ptr_target = if has_overflow { 20 - overflow_marker.len() } else { 20 };
        while ptr < ptr_target {
            disp.print_char('=');
            ptr += 1;
        }

        if has_overflow {
            disp.print_string(overflow_marker);
        }
    }

    fn draw_expression(&mut self) {
        // Try to parse and get warning spans
        let (parser, _) = self.parse();
        let warning_indices = parser.constant_overflow_spans.iter()
            .map(|s| s.indices().collect::<Vec<_>>())
            .flatten().collect::<Vec<_>>();

        self.constant_overflows = !warning_indices.is_empty();
        
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
                disp.print_cursor_left()
            } else if i == self.cursor_pos {
                disp.print_cursor_right()
            } else {
                if warning_indices.contains(&i) {
                    disp.print_char('!')
                } else {
                    disp.print_char(' ')
                }
            }
        }
    }

    fn draw_result(&mut self) {
        let disp = self.hal.display_mut();

        disp.set_position(0, 3);
        if let Some(result) = &self.eval_result {
            match result {
                Ok(result) => {
                    let str = if self.eval_config.data_type.signed {
                        todo!("unsigned string not implemented")
                    } else {
                        result.result.to_unsigned_decimal_string()
                    };
                    disp.print_string(&str);
                },
                Err(_) => disp.print_string("parse error"),
            }
        } else {
            disp.print_string(&str::repeat(" ", 20));
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
                    self.clear_evaluation();
                }
            },
            Key::Exe => {
                self.evaluate();
                self.draw_result();
                self.draw_header();
            }

            Key::Shift => (),
            Key::Menu => (),
        }
    }

    fn insert_and_redraw(&mut self, glyph: Glyph) {
        self.glyphs.insert(self.cursor_pos, glyph);
        self.cursor_pos += 1;
        self.draw_expression();
        self.clear_evaluation();
    }

    fn parse(&self) -> (Parser, Result<Node, ParserError>) {
        let mut parser = Parser::new(&self.glyphs, self.eval_config);
        let result = parser.parse();
        (parser, result)
    }

    fn evaluate(&mut self) {
        let (_, node) = self.parse();
        self.eval_result = Some(node.map(|node| evaluate(&node, &self.eval_config)))
    }

    fn clear_evaluation(&mut self) {
        self.eval_result = None;
        self.draw_result();
        self.draw_header();
    }
}
