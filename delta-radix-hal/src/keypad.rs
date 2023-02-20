use async_trait::async_trait;
use alloc::boxed::Box;

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

    Left,
    Right,
    Delete,

    HexBase,
    BinaryBase,

    FormatSelect,

    DebugTerminate,
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
        }
    }

    pub fn from_u32(key: u32) -> Key {
        match key {
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

            _ => panic!("no such key"),
        }
    }
}

#[async_trait(?Send)]
pub trait Keypad {
    async fn wait_key(&mut self) -> Key;
}
