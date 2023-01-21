#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Base {
    Decimal,
    Hexadecimal,
    Binary,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Glyph {
    Digit(u8),

    Add,
    Subtract,
    Multiply,
    Divide,

    Base(Base),
}

impl Glyph {
    pub fn to_char(self) -> char {
        match self {
            Glyph::Digit(d) => char::from_digit(d as u32, 10).unwrap(),

            Glyph::Add => '+',
            Glyph::Subtract => '-',
            Glyph::Multiply => '×',
            Glyph::Divide => '÷',

            Glyph::Base(Base::Hexadecimal) => 'x',
            Glyph::Base(Base::Binary) => 'b',
            Glyph::Base(Base::Decimal) => 'd',
        }
    }
}
