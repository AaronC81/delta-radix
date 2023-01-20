#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Glyph {
    Digit(u8),
    HexBase,
    BinaryBase,
}

impl Glyph {
    pub fn to_char(self) -> char {
        match self {
            Glyph::Digit(d) => char::from_digit(d as u32, 10).unwrap(),
            Glyph::HexBase => 'x',
            Glyph::BinaryBase => 'b',
        }
    }
}
