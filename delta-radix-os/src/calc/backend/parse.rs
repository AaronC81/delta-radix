use core::{ops::Range, marker::PhantomData};

use alloc::{vec, vec::Vec, string::{String, ToString}, boxed::Box, format};
use delta_radix_hal::Glyph;

use super::eval;
use crate::calc::frontend::{Base, VariableArray};
use flex_int::FlexInt;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct GlyphSpan {
    start: usize,
    length: usize,
}

impl GlyphSpan {
    pub fn indices(&self) -> Range<usize> {
        self.start..(self.start + self.length)
    }

    pub fn end(&self) -> usize {
        self.start + self.length - 1
    }

    pub fn merge(&self, other: Self) -> Self {
        let new_start = self.start.min(other.start);
        let new_end = self.end().max(other.end());
        let new_length = new_end - new_start + 1;

        GlyphSpan { start: new_start, length: new_length }
    }
}

pub struct Node {
    span: GlyphSpan,
    pub kind: NodeKind,
}

pub enum NodeKind {
    Number(FlexInt),

    Add(Box<Node>, Box<Node>),
    Subtract(Box<Node>, Box<Node>),
    Divide(Box<Node>, Box<Node>),
    Multiply(Box<Node>, Box<Node>),
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ParserError {
    ptr: usize,
    kind: ParserErrorKind,
}

impl ParserError {
    pub fn describe(&self) -> String {
        self.kind.describe()
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum ParserErrorKind {
    DuplicateBase,
    InvalidNumber,
    UnexpectedGlyph(Glyph),
    ExpectedParen,
    UnexpectedEnd,
    InvalidVariable,
}

impl ParserErrorKind {
    pub fn describe(&self) -> String {
        match self {
            ParserErrorKind::DuplicateBase => "duplicate base".to_string(),
            ParserErrorKind::InvalidNumber => "invalid number".to_string(),
            ParserErrorKind::UnexpectedGlyph(g) => format!("unexpected {}", g.describe()),
            ParserErrorKind::ExpectedParen => "expected paren".to_string(),
            ParserErrorKind::UnexpectedEnd => "unexpected end".to_string(),
            ParserErrorKind::InvalidVariable => "invalid variable".to_string(),
        }
    }
}

pub struct Parser<'g, 'v, N: NumberParser> {
    pub glyphs: &'g [Glyph],
    pub variables: &'v VariableArray,
    pub ptr: usize,
    pub eval_config: eval::Configuration,
    pub constant_overflow_spans: Vec<GlyphSpan>,
    pub next_number_unary_negations: usize,

    _phantom: PhantomData<N>,
}

impl<'g, 'v, N: NumberParser> Parser<'g, 'v, N> {
    pub fn new(glyphs: &'g [Glyph], variables: &'v VariableArray, eval_config: eval::Configuration) -> Self {
        Parser {
            glyphs,
            variables,
            ptr: 0,
            eval_config,
            constant_overflow_spans: vec![],
            next_number_unary_negations: 0,

            _phantom: PhantomData,
        }
    }

    pub fn parse(&mut self) -> Result<Node, ParserError> {
        // Special case - if there are no tokens, parse to 0
        if self.glyphs.is_empty() {
            return Ok(Node {
                span: GlyphSpan { start: 0, length: 0 },
                kind: NodeKind::Number(FlexInt::new(self.eval_config.data_type.bits)),
            })
        }

        let result = self.parse_top_level()?;

        // Check we reached the end
        if let Some(glyph) = self.here() {
            Err(self.create_error(ParserErrorKind::UnexpectedGlyph(glyph)))
        } else {
            Ok(result)
        }
    }

    fn here(&self) -> Option<Glyph> {
        self.glyphs.get(self.ptr).copied()
    }

    fn peek(&self) -> Option<Glyph> {
        self.glyphs.get(self.ptr + 1).copied()
    }

    fn advance(&mut self) {
        self.ptr += 1;
    }

    fn parse_top_level(&mut self) -> Result<Node, ParserError> {
        self.parse_add_sub()
    }

    fn parse_add_sub(&mut self) -> Result<Node, ParserError> {
        let mut current = self.parse_mul_div()?;

        while let Some(op @ (Glyph::Add | Glyph::Subtract)) = self.here() {
            self.advance();
            let rhs = self.parse_mul_div()?;
            let span = current.span.merge(rhs.span);
            let kind = match op {
                Glyph::Add => NodeKind::Add(Box::new(current), Box::new(rhs)),
                Glyph::Subtract => NodeKind::Subtract(Box::new(current), Box::new(rhs)),
                _ => unreachable!(),
            };
            current = Node { span, kind };
        }

        Ok(current)
    }

    fn parse_mul_div(&mut self) -> Result<Node, ParserError> {
        let mut current = self.parse_bottom()?;

        while let Some(op @ (Glyph::Multiply | Glyph::Divide)) = self.here() {
            self.advance();
            let rhs = self.parse_bottom()?;
            let span = current.span.merge(rhs.span);
            let kind = match op {
                Glyph::Multiply => NodeKind::Multiply(Box::new(current), Box::new(rhs)),
                Glyph::Divide => NodeKind::Divide(Box::new(current), Box::new(rhs)),
                _ => unreachable!(),
            };
            current = Node { span, kind };
        }

        Ok(current)
    }

    fn parse_bottom(&mut self) -> Result<Node, ParserError> {
        // Subtract as negation
        if let Some(Glyph::Subtract) = self.here() {
            self.next_number_unary_negations += 1;
            self.advance();
            return self.parse_bottom();
        }

        // Check for parentheses
        if let Some(Glyph::LeftParen) = self.here() {
            self.advance();
            let node = self.parse_top_level()?;
            let Some(Glyph::RightParen) = self.here() else {
                return Err(self.create_error(ParserErrorKind::ExpectedParen.into()))
            };
            self.advance();

            return Ok(node);
        }

        // Check for variable
        if let Some(Glyph::Variable) = self.here() {
            // Figure out which variable we're using
            self.advance();
            let Some(Glyph::Digit(d)) = self.here() else {
                return Err(self.create_error(ParserErrorKind::InvalidVariable.into()))
            };
            if d as usize >= self.variables.len() {
                return Err(self.create_error(ParserErrorKind::InvalidVariable.into()))
            };
            self.advance();

            // Parse its contents
            let variable_glyphs = &self.variables[d as usize];
            let mut variable_parser = Parser::<N>::new(
                &variable_glyphs,
                self.variables,
                self.eval_config,
            );
            let variable_node = variable_parser.parse()?;

            if !variable_parser.constant_overflow_spans.is_empty() {
                self.constant_overflow_spans.push(GlyphSpan {
                    start: self.ptr - 2, length: 2,
                })
            }

            return Ok(variable_node);
        }

        // Number
        if let Some(g @ (Glyph::Digit(_) | Glyph::HexBase | Glyph::BinaryBase | Glyph::DecimalBase)) = self.here() {
            let mut start = self.ptr;
            let mut digits = vec![];
            let mut base = None;

            // Check for base at start
            if let Some(b) = Base::from_glyph(g) {
                self.advance();
                base = Some(b);
            };

            // Gather digits
            while let Some(Glyph::Digit(d)) = self.here() {
                digits.push(char::from_digit(d as u32, 16).unwrap());
                self.advance();
            }

            // Check for base at end
            if let Some(b) = self.here().map(Base::from_glyph).flatten() {
                if base.is_some() {
                    return Err(self.create_error(ParserErrorKind::DuplicateBase));
                }
                self.advance();
                base = Some(b);
            };

            // Construct string of digits, considering negation
            // (Specifically we want an odd number of unary negations; -2 is negative, --2 isn't)
            let mut str: String = digits.into_iter().collect();
            let mut force_parse_signed = false;
            if self.next_number_unary_negations % 2 == 1 {
                start -= self.next_number_unary_negations;
                str.insert(0, '-');
                self.next_number_unary_negations = 0;

                // We'll need to parse this number as signed, even though the underlying data type
                // is unsigned
                if !self.eval_config.data_type.signed {
                    force_parse_signed = true;
                }
            }

            // Parse number
            let parse_signed = self.eval_config.data_type.signed || force_parse_signed;
            let (num, mut overflow) =
                N::parse(&str, base.unwrap_or(Base::Decimal), parse_signed, self.eval_config.data_type.bits)
                .ok_or(self.create_error(ParserErrorKind::InvalidNumber))?;

            // Force-parsing a negative number will always result in overflow (because the data type
            // can't represent the parsed number)
            if force_parse_signed {
                overflow = true;
            }

            // Add warning region of number parsing overflowed
            let length = self.ptr - start;
            let span = GlyphSpan { start, length };
            if overflow {
                self.constant_overflow_spans.push(span);
            }

            Ok(Node { span, kind: NodeKind::Number(num) })
        } else if let Some(glyph) = self.here() {
            Err(self.create_error(ParserErrorKind::UnexpectedGlyph(glyph)))
        } else {
            Err(self.create_error(ParserErrorKind::UnexpectedEnd))
        }
    }

    fn create_error(&self, kind: ParserErrorKind) -> ParserError {
        ParserError { ptr: self.ptr, kind }
    }
}

pub trait NumberParser {
    fn parse(chars: &str, base: Base, signed: bool, bits: usize) -> Option<(FlexInt, bool)>;
}

impl NumberParser for FlexInt {
    fn parse(chars: &str, base: Base, signed: bool, bits: usize) -> Option<(FlexInt, bool)> {
        match base {
            Base::Decimal => 
                if signed {
                    FlexInt::from_signed_decimal_string(chars, bits)
                } else {
                    FlexInt::from_unsigned_decimal_string(chars, bits)
                }
            Base::Hexadecimal =>
                if signed {
                    FlexInt::from_signed_hex_string(chars, bits)
                } else {
                    FlexInt::from_unsigned_hex_string(chars, bits)
                }
            Base::Binary => 
                if signed {
                    FlexInt::from_signed_binary_string(chars, bits)
                } else {
                    FlexInt::from_unsigned_binary_string(chars, bits)
                }
        }
    }
}

/// A [NumberParser] implementation which always returns a garbage FlexInt but does accurately
/// capture overflow. It is that is significantly faster than the implementation on [FlexInt],
/// suitable for per-keypress constant overflow checking.
pub struct ConstantOverflowChecker;
impl NumberParser for ConstantOverflowChecker {
    fn parse(chars: &str, base: Base, signed: bool, bits: usize) -> Option<(FlexInt, bool)> {
        let Ok(num) = i128::from_str_radix(chars, base.radix()) else {
            // To play it safe, treat parse errors as constant overflow
            // (otherwise, ludicrously large numbers may overflow)
            return Some((FlexInt::new(1), true));
        };
        let overflow = if signed {
            num >= 2_i128.pow(bits as u32 - 1) || num < -1 * 2_i128.pow(bits as u32 - 1)
        } else {
            num >= 2_i128.pow(bits as u32)
        };
        Some((FlexInt::new(1), overflow))
    }
}
