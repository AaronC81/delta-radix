use core::ops::Range;

use alloc::{vec, vec::Vec, string::{String, ToString}, boxed::Box, format};
use delta_radix_hal::Glyph;

use super::{eval, Base};
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
    UnexpectedEnd,
}

impl ParserErrorKind {
    pub fn describe(&self) -> String {
        match self {
            ParserErrorKind::DuplicateBase => "duplicate base".to_string(),
            ParserErrorKind::InvalidNumber => "invalid number".to_string(),
            ParserErrorKind::UnexpectedGlyph(g) => format!("unexpected {}", g.describe()),
            ParserErrorKind::UnexpectedEnd => "unexpected end".to_string(),
        }
    }
}

pub struct Parser<'g> {
    pub glyphs: &'g [Glyph],
    pub ptr: usize,
    pub eval_config: eval::Configuration,
    pub constant_overflow_spans: Vec<GlyphSpan>,
    pub next_number_negated: bool,
}

impl<'g> Parser<'g> {
    pub fn new(glyphs: &'g [Glyph], eval_config: eval::Configuration) -> Self {
        Parser {
            glyphs,
            ptr: 0,
            eval_config,
            constant_overflow_spans: vec![],
            next_number_negated: false,
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
            self.next_number_negated = !self.next_number_negated;
            self.advance();
            return self.parse_bottom();
        }

        // Number
        if let Some(g @ (Glyph::Digit(_) | Glyph::HexBase | Glyph::BinaryBase | Glyph::DecimalBase)) = self.here() {
            let start = self.ptr;
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
            let mut str: String = digits.into_iter().collect();
            let mut force_parse_signed = false;
            if self.next_number_negated {
                str.insert(0, '-');
                self.next_number_negated = false;

                // We'll need to parse this number as signed, even though the underlying data type
                // is unsigned
                if !self.eval_config.data_type.signed {
                    force_parse_signed = true;
                }
            }

            // Parse number
            let parse_signed = self.eval_config.data_type.signed || force_parse_signed;
            let (num, mut overflow) =
                match base {
                    Some(Base::Decimal) | None => 
                        if parse_signed {
                            FlexInt::from_signed_decimal_string(&str, self.eval_config.data_type.bits)
                        } else {
                            FlexInt::from_unsigned_decimal_string(&str, self.eval_config.data_type.bits)
                        }
                    Some(Base::Hexadecimal) =>
                        if parse_signed {
                            FlexInt::from_signed_hex_string(&str, self.eval_config.data_type.bits)
                        } else {
                            FlexInt::from_unsigned_hex_string(&str, self.eval_config.data_type.bits)
                        }
                    _ => todo!("base not yet implemented"),
                }
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
