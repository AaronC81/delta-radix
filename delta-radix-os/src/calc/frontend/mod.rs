use alloc::{vec::Vec, vec, string::{ToString, String}, format};
use delta_radix_hal::{Hal, Display, Keypad, Key, DisplaySpecialCharacter, Glyph};
use flex_int::FlexInt;

use crate::calc::backend::{eval::{EvaluationResult, Configuration, DataType, evaluate}, parse::{Parser, Node, ParserError, NumberParser, ConstantOverflowChecker}};

mod draw;
mod input;

#[derive(PartialEq, Eq, Clone, Debug)]
enum ApplicationState {
    Normal,
    OutputBaseSelect,
    FormatMenu {
        bits_digits: String,
        bits_cursor_pos: usize,
    },
    OutputSignedMenu,
    VariableSet,
    VariableView {
        page: u8,
    },
    MainMenu,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Base {
    Decimal,
    Hexadecimal,
    Binary,
}

impl Base {
    pub fn from_glyph(glyph: Glyph) -> Option<Self> {
        match glyph {
            Glyph::HexBase => Some(Base::Hexadecimal),
            Glyph::BinaryBase => Some(Base::Binary),
            Glyph::DecimalBase => Some(Base::Decimal),
            _ => None,
        }
    }
    
    pub fn radix(&self) -> u32 {
        match self {
            Base::Decimal => 10,
            Base::Hexadecimal => 16,
            Base::Binary => 2,
        }
    }
}

// Variables are stored as sequences of glyphs rather than FlexInts, so that they continue working
// across changes in data type
pub type VariableArray = [Vec<Glyph>; 16];

pub struct CalculatorApplication<'h, H: Hal> {
    hal: &'h mut H,

    state: ApplicationState,
    input_shifted: bool,

    output_format: Base,
    signed_result: Option<bool>,

    glyphs: Vec<Glyph>,
    cursor_pos: usize,
    constant_overflows: bool,
    scroll_offset: usize,

    eval_config: Configuration,
    eval_result: Option<Result<EvaluationResult, ParserError>>,

    variables: VariableArray,
}

impl<'h, H: Hal> CalculatorApplication<'h, H> {
    pub const WIDTH: usize = 20;

    pub fn new(hal: &'h mut H) -> Self {
        Self {
            hal,
            state: ApplicationState::Normal,
            output_format: Base::Decimal,
            signed_result: None,
            input_shifted: false,
            glyphs: vec![],
            cursor_pos: 0,
            scroll_offset: 0,
            eval_config: Configuration {
                data_type: DataType {
                    bits: 32,
                    signed: false,
                }
            },
            eval_result: None,
            constant_overflows: false,

            // Variables are initially 0
            variables: (0..16).into_iter()
                .map(|_| vec![Glyph::Digit(0)])
                .collect::<Vec<_>>().try_into().unwrap()
        }
    }

    pub async fn main(&mut self) {
        self.draw_full();

        loop {
            let key = self.hal.keypad_mut().wait_key().await;
            self.process_input_and_redraw(key).await;
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
        let mut parser = Parser::new(&self.glyphs, &self.variables, self.eval_config);
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

    fn eval_result_to_string(&self) -> Option<String> {
        let Some(ref result) = self.eval_result else { return None };

        Some(match result {
            Ok(result) => {
                let signed = self.signed_result.unwrap_or(self.eval_config.data_type.signed);
                match self.output_format {
                    Base::Decimal => {
                        if signed {
                            result.result.to_signed_decimal_string()
                        } else {
                            result.result.to_unsigned_decimal_string()
                        }
                    }
                    Base::Hexadecimal => {
                        format!("x{}", if signed {
                            result.result.to_signed_hex_string()
                        } else {
                            result.result.to_unsigned_hex_string()
                        })
                    }
                    Base::Binary => {
                        format!("b{}", if signed {
                            result.result.to_signed_binary_string()
                        } else {
                            result.result.to_unsigned_binary_string()
                        })
                    }
                }
                
            },
            Err(e) => e.describe(),
        })
    }
}
