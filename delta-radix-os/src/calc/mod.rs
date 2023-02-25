use alloc::{vec::Vec, vec, string::{ToString, String}, format};
use delta_radix_hal::{Hal, Display, Keypad, Key, DisplaySpecialCharacter, Glyph};
use flex_int::FlexInt;

use crate::menu;

use self::{eval::{EvaluationResult, Configuration, DataType, evaluate}, parse::{Parser, Node, ParserError, NumberParser, ConstantOverflowChecker}};

mod eval;
mod parse;

#[derive(PartialEq, Eq, Clone, Debug)]
enum ApplicationState {
    Normal,
    OutputBaseSelect,
    FormatMenu {
        bits_digits: String,
        bits_cursor_pos: usize,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Base {
    Decimal,
    Hexadecimal,
    Binary,
}

impl Base {
    fn from_glyph(glyph: Glyph) -> Option<Self> {
        match glyph {
            Glyph::HexBase => Some(Base::Hexadecimal),
            Glyph::BinaryBase => Some(Base::Binary),
            Glyph::DecimalBase => Some(Base::Decimal),
            _ => None,
        }
    }
    
    fn radix(&self) -> u32 {
        match self {
            Base::Decimal => 10,
            Base::Hexadecimal => 16,
            Base::Binary => 2,
        }
    }
}

pub struct CalculatorApplication<'h, H: Hal> {
    hal: &'h mut H,

    state: ApplicationState,
    output_format: Base,
    input_shifted: bool,

    glyphs: Vec<Glyph>,
    cursor_pos: usize,
    constant_overflows: bool,
    scroll_offset: usize,

    eval_config: Configuration,
    eval_result: Option<Result<EvaluationResult, ParserError>>,
}

impl<'h, H: Hal> CalculatorApplication<'h, H> {
    pub const WIDTH: usize = 20;

    pub fn new(hal: &'h mut H) -> Self {
        Self {
            hal,
            state: ApplicationState::Normal,
            output_format: Base::Decimal,
            input_shifted: false,
            glyphs: vec![],
            cursor_pos: 0,
            scroll_offset: 0,
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
            self.process_input_and_redraw(key).await;
        }
    }

    fn draw_full(&mut self) {
        self.hal.display_mut().clear();
        match self.state {
            ApplicationState::Normal | ApplicationState::OutputBaseSelect => {
                self.draw_header();
                self.draw_expression();
                self.draw_result();
            }

            ApplicationState::FormatMenu { ref bits_digits, bits_cursor_pos } => {
                let display = self.hal.display_mut();
                let bits_header = "Bits: ";

                display.set_position((bits_header.len() as u8 + bits_cursor_pos as u8) - 1, 0);
                display.print_special(DisplaySpecialCharacter::CursorLeft);
                display.print_special(DisplaySpecialCharacter::CursorRight);

                display.set_position(0, 1);
                display.print_string(bits_header);
                display.print_string(bits_digits);

                display.set_position(0, 2);
                display.print_string("-) Signed  ");
                if self.eval_config.data_type.signed {
                    display.print_string(" <");
                }
                display.set_position(0, 3);
                display.print_string("+) Unsigned");
                if !self.eval_config.data_type.signed {
                    display.print_string(" <");
                }
            }
        }
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
        let ptr_target = if has_overflow { Self::WIDTH - overflow_marker.len() } else { Self::WIDTH };
        while ptr < ptr_target {
            if self.input_shifted {
                disp.print_char('^');
            } else {
                disp.print_char('=');
            }
            ptr += 1;
        }

        if has_overflow {
            disp.print_string(overflow_marker);
        }
    }

    fn draw_expression(&mut self) {
        self.adjust_scroll();

        // Try to parse and get warning spans
        let (parser, _) = self.parse::<ConstantOverflowChecker>();
        let warning_indices = parser.constant_overflow_spans.iter()
            .flat_map(|s| s.indices().collect::<Vec<_>>())
            .collect::<Vec<_>>();

        self.constant_overflows = !warning_indices.is_empty();
        
        let disp = self.hal.display_mut();

        // Draw expression
        disp.set_position(0, 2);
        let mut chars_written = 0;
        for glyph in self.glyphs.iter().skip(self.scroll_offset).take(Self::WIDTH) {
            disp.print_glyph(*glyph);
            chars_written += 1;
        }
        for _ in chars_written..Self::WIDTH {
            disp.print_char(' ');
        }

        // Draw cursor
        disp.set_position(0, 1);
        for i in self.scroll_offset..(self.scroll_offset + Self::WIDTH) {
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

        if self.state == ApplicationState::OutputBaseSelect {
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
                            Base::Hexadecimal => {
                                str = format!("x{}", if self.eval_config.data_type.signed {
                                    result.result.to_signed_hex_string()
                                } else {
                                    result.result.to_unsigned_hex_string()
                                });
                            }
                            Base::Binary => {
                                str = format!("b{}", if self.eval_config.data_type.signed {
                                    result.result.to_signed_binary_string()
                                } else {
                                    result.result.to_unsigned_binary_string()
                                });
                            }
                        }
                        
                    },
                    Err(e) => str = e.describe(),
                }
            } else {
                str = str::repeat(" ", Self::WIDTH);
            }
        }

        disp.set_position((Self::WIDTH - str.len()) as u8, 3);
        disp.print_string(&str);
    }

    async fn process_input_and_redraw(&mut self, key: Key) {
        if menu::check_menu(self.hal, key, self.input_shifted).await {
            self.draw_full();
            return
        }

        match self.state {
            ApplicationState::Normal =>
                if self.input_shifted {
                    match key {
                        Key::Shift => {
                            self.input_shifted = false;
                            self.draw_header();
                        }
                        Key::Delete => {
                            self.clear_all(true);
                            self.draw_full();
                        }

                        _ => (),
                    }
                } else {
                    match key {
                        Key::Digit(d) => self.insert_and_redraw(Glyph::Digit(d)),
                        Key::HexBase => self.insert_and_redraw(Glyph::HexBase),
                        Key::BinaryBase => self.insert_and_redraw(Glyph::BinaryBase),
            
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
                                self.clear_evaluation(true);
                            }
                        },
                        Key::Exe => {
                            self.evaluate();
                            self.draw_result();
                            self.draw_header();
                        }

                        Key::FormatSelect => {
                            self.state = ApplicationState::OutputBaseSelect;
                            self.draw_result();
                        }
            
                        Key::Shift => {
                            self.input_shifted = true;
                            self.draw_header();
                        }

                        Key::Sleep => {
                            // Do not redraw - the HAL is expected to deal with this
                            self.clear_all(false);
                        }

                        Key::Menu => {
                            let bits_digits = self.eval_config.data_type.bits.to_string();
                            self.state = ApplicationState::FormatMenu {
                                bits_cursor_pos: bits_digits.len(),
                                bits_digits,
                            };
                            self.draw_full();
                        }
                        
                        Key::DebugTerminate => (),
                    }
                },
            
            ApplicationState::OutputBaseSelect => match key {
                Key::HexBase => self.set_output_format_and_redraw(Base::Hexadecimal),
                Key::BinaryBase => self.set_output_format_and_redraw(Base::Binary),
                Key::FormatSelect => self.set_output_format_and_redraw(Base::Decimal),

                _ => (),
            }

            ApplicationState::FormatMenu { ref mut bits_digits, ref mut bits_cursor_pos } => match key {
                Key::Digit(d) => {
                    bits_digits.push(char::from_digit(d as u32, 10).unwrap());
                    *bits_cursor_pos += 1;
                    self.draw_full();
                }

                Key::Delete => {
                    if *bits_cursor_pos > 0 {
                        bits_digits.remove(*bits_cursor_pos - 1);
                        *bits_cursor_pos -= 1;
                        self.draw_full();
                    }
                }
                Key::Left => {
                    if *bits_cursor_pos > 0 {
                        *bits_cursor_pos -= 1;
                        self.draw_full();
                    }
                }
                Key::Right => {
                    if *bits_cursor_pos < bits_digits.len() {
                        *bits_cursor_pos += 1;
                        self.draw_full();
                    }
                }

                Key::Add => {
                    self.eval_config.data_type.signed = false;
                    self.draw_full();
                }
                Key::Subtract => {
                    self.eval_config.data_type.signed = true;
                    self.draw_full();
                }

                Key::FormatSelect | Key::Menu | Key::Exe => {
                    // Apply bits evaluation settings
                    if let Ok(mut bits) = bits_digits.parse() {
                        // Minimum supported number of bits
                        if bits < 3 {
                            bits = 3;
                        }

                        self.eval_config.data_type.bits = bits;
                    }

                    self.state = ApplicationState::Normal;
                    self.clear_evaluation(true);
                    self.draw_full();
                }

                _ => (),
            }
        }
        
    }

    fn insert_and_redraw(&mut self, glyph: Glyph) {
        self.glyphs.insert(self.cursor_pos, glyph);
        self.cursor_pos += 1;
        self.draw_expression();
        self.clear_evaluation(true);
    }

    fn set_output_format_and_redraw(&mut self, base: Base) {
        self.output_format = base;
        self.state = ApplicationState::Normal;
        self.draw_full();
    }

    fn parse<N: NumberParser>(&self) -> (Parser<N>, Result<Node, ParserError>) {
        let mut parser = Parser::new(&self.glyphs, self.eval_config);
        let result = parser.parse();
        (parser, result)
    }

    fn evaluate(&mut self) {
        let (_, node) = self.parse::<FlexInt>();
        self.eval_result = Some(node.map(|node| evaluate(&node, &self.eval_config)))
    }

    fn clear_evaluation(&mut self, redraw: bool) {
        self.eval_result = None;

        if redraw {
            self.draw_result();
            self.draw_header();
        }
    }

    fn clear_all(&mut self, redraw: bool) {
        self.clear_evaluation(redraw);
        self.glyphs.clear();
        self.cursor_pos = 0;
        self.scroll_offset = 0;
        self.input_shifted = false;
    }

    fn adjust_scroll(&mut self) {
        // Check if we need to scroll to the left
        if self.cursor_pos == self.scroll_offset && self.cursor_pos > 0 {
            self.scroll_offset -= 1;
        }

        // Check if we need to scroll to the right
        if self.cursor_pos == self.scroll_offset + Self::WIDTH {
            self.scroll_offset += 1;
        }
    }
}
