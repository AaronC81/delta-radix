use alloc::{vec::Vec, vec, string::ToString};
use delta_radix_hal::{Hal, Display, Keypad, Key, DisplaySpecialCharacter};

mod glyph;
use self::{glyph::{Glyph, Base}, eval::{EvaluationResult, Configuration, DataType, evaluate}, parse::{Parser, Node, ParserError}};

mod eval;
mod parse;

#[derive(PartialEq, Eq, Clone, Debug)]
enum ApplicationState {
    Normal,
    FormatSelect,
}

pub struct CalculatorApplication<'h, H: Hal> {
    hal: &'h mut H,

    state: ApplicationState,
    output_format: Base,

    glyphs: Vec<Glyph>,
    cursor_pos: usize,
    constant_overflows: bool,

    eval_config: Configuration,
    eval_result: Option<Result<EvaluationResult, ParserError>>,
}

impl<'h, H: Hal> CalculatorApplication<'h, H> {
    pub fn new(hal: &'h mut H) -> Self {
        Self {
            hal,
            state: ApplicationState::Normal,
            output_format: Base::Decimal,
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
            let warn = warning_indices.contains(&i);
            if i + 1 == self.cursor_pos {
                if warn {
                    disp.print_special(DisplaySpecialCharacter::CursorLeftWithWarning)
                } else {
                    disp.print_special(DisplaySpecialCharacter::CursorLeft)
                }
            } else if i == self.cursor_pos {
                if warn {
                    disp.print_special(DisplaySpecialCharacter::CursorRightWithWarning)
                } else {
                    disp.print_special(DisplaySpecialCharacter::CursorRight)
                }
            } else {
                if warn {
                    disp.print_special(DisplaySpecialCharacter::Warning)
                } else {
                    disp.print_char(' ')
                }
            }
        }
    }

    fn draw_result(&mut self) {
        let disp = self.hal.display_mut();

        let str;

        if self.state == ApplicationState::FormatSelect {
            str = "BASE?".to_string();
        } else {
            if let Some(result) = &self.eval_result {
                match result {
                    Ok(result) => {
                        match self.output_format {
                            Base::Decimal => {
                                str = if self.eval_config.data_type.signed {
                                    result.result.to_signed_decimal_string()
                                } else {
                                    result.result.to_unsigned_decimal_string()
                                };
                            }
                            Base::Hexadecimal => todo!(),
                            Base::Binary => todo!(),
                        }
                        
                    },
                    Err(_) => str = "parse error".to_string(),
                }
            } else {
                str = str::repeat(" ", 20);
            }
        }

        disp.set_position(20 - str.len() as u8, 3);
        disp.print_string(&str);
    }

    fn process_input_and_redraw(&mut self, key: Key) {
        match self.state {
            ApplicationState::Normal => match key {
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

                Key::FormatSelect => {
                    self.state = ApplicationState::FormatSelect;
                    self.draw_result();
                }
    
                Key::Shift => (),
                Key::Menu => (),
            },
            
            ApplicationState::FormatSelect => match key {
                Key::HexBase => self.set_output_format_and_redraw(Base::Hexadecimal),
                Key::BinaryBase => self.set_output_format_and_redraw(Base::Binary),
                Key::FormatSelect => self.set_output_format_and_redraw(Base::Decimal),

                _ => (),
            }
        }
        
    }

    fn insert_and_redraw(&mut self, glyph: Glyph) {
        self.glyphs.insert(self.cursor_pos, glyph);
        self.cursor_pos += 1;
        self.draw_expression();
        self.clear_evaluation();
    }

    fn set_output_format_and_redraw(&mut self, base: Base) {
        self.output_format = base;
        self.state = ApplicationState::Normal;
        self.draw_full();
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
