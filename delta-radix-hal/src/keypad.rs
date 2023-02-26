#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Key {
    Digit(u8),
    Shift,
    Menu,
    Exe,

    Add,
    Subtract,
    Multiply,
    Divide,

    Variable,

    Left,
    Right,
    Delete,

    HexBase,
    BinaryBase,

    FormatSelect,

    // Neither are actual keys, just markers to communicate things to OS
    DebugTerminate,
    Sleep,
}

impl Key {
    pub fn to_u32(&self) -> u32 {
        match self {
            Key::Digit(d) => *d as u32,
            Key::Shift => 0x101,
            Key::Menu => 0x102,
            Key::Exe => 0x103,
            Key::Add => 0x104,
            Key::Subtract => 0x105,
            Key::Multiply => 0x106,
            Key::Divide => 0x107,
            Key::Left => 0x108,
            Key::Right => 0x109,
            Key::Delete => 0x10A,
            Key::HexBase => 0x10B,
            Key::BinaryBase => 0x10C,
            Key::FormatSelect => 0x10D,
            Key::DebugTerminate => 0x10E,
            Key::Sleep => 0x10F,
            Key::Variable => 0x110,
        }
    }

    pub fn from_u32(key: u32) -> Option<Key> {
        Some(match key {
            d if d < 0x100 => Key::Digit(d as u8),
            0x101 => Key::Shift,
            0x102 => Key::Menu,
            0x103 => Key::Exe,
            0x104 => Key::Add,
            0x105 => Key::Subtract,
            0x106 => Key::Multiply,
            0x107 => Key::Divide,
            0x108 => Key::Left,
            0x109 => Key::Right,
            0x10A => Key::Delete,
            0x10B => Key::HexBase,
            0x10C => Key::BinaryBase,
            0x10D => Key::FormatSelect,
            0x10E => Key::DebugTerminate,
            0x10F => Key::Sleep,
            0x110 => Key::Variable,

            _ => return None,
        })
    }
}

pub trait Keypad {
    async fn wait_key(&mut self) -> Key;
}
