use core::ops::Range;

use alloc::{vec, vec::Vec, string::String, boxed::Box};

use super::{num::FlexInt, glyph::{Glyph, Base}, eval};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct GlyphSpan {
    start: usize,
    length: usize,
}

impl GlyphSpan {
    pub fn indices(&self) -> Range<usize> {
        self.start..(self.start + self.length)
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

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum ParserErrorKind {
    DuplicateBase,
    UnexpectedGlyph(Glyph),
    UnexpectedEnd,
}

pub struct Parser<'g> {
    pub glyphs: &'g [Glyph],
    pub ptr: usize,
    pub eval_config: eval::Configuration,
    pub constant_overflow_spans: Vec<GlyphSpan>,
}

impl<'g> Parser<'g> {
    pub fn new(glyphs: &'g [Glyph], eval_config: eval::Configuration) -> Self {
        Parser {
            glyphs,
            ptr: 0,
            eval_config,
            constant_overflow_spans: vec![],
        }
    }

    pub fn parse(&mut self) -> Result<Node, ParserError> {
        self.parse_bottom()
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

    fn parse_bottom(&mut self) -> Result<Node, ParserError> {
        if let Some(Glyph::Digit(_) | Glyph::Base(_)) = self.here() {
            let start = self.ptr;
            let mut digits = vec![];
            let mut base = None;

            // Check for base at start
            if let Some(Glyph::Base(b)) = self.here() {
                self.advance();
                base = Some(b);
            };

            // Gather digits
            loop {
                match self.here() {
                    Some(Glyph::Digit(d)) => {
                        digits.push(char::from_digit(d as u32, 10).unwrap());
                        self.advance();
                    },
                    _ => break,
                }
            }

            // Check for base at end
            if let Some(Glyph::Base(b)) = self.here() {
                if base.is_some() {
                    return Err(self.create_error(ParserErrorKind::DuplicateBase));
                }
                self.advance();
                base = Some(b);
            };

            // Construct string of digits and parse number
            let str: String = digits.into_iter().collect();
            let (num, overflow) = match base {
                Some(Base::Decimal) | None => FlexInt::from_decimal_string(&str, self.eval_config.data_type.bits),
                Some(Base::Hexadecimal) => FlexInt::from_hex_string(&str, self.eval_config.data_type.bits),
                _ => todo!("base not yet implemented"),
            };

            // Add warning region of number parsing overflowed
            let length = self.ptr - start;
            let span = GlyphSpan { start, length };
            if overflow {
                self.constant_overflow_spans.push(span);
            }

            Ok(Node { span, kind: NodeKind::Number(num) })
        } else {
            if let Some(glyph) = self.here() {
                Err(self.create_error(ParserErrorKind::UnexpectedGlyph(glyph)))
            } else {
                Err(self.create_error(ParserErrorKind::UnexpectedEnd))
            }
        }
    }

    fn create_error(&self, kind: ParserErrorKind) -> ParserError {
        ParserError { ptr: self.ptr, kind }
    }
}
