use async_trait::async_trait;
use alloc::boxed::Box;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Key {
    Digit(u8),
    Shift,
    Menu,

    Add,
    Subtract,
    Multiply,
    Divide,

    Left,
    Right,
    Delete,

    HexBase,
    BinaryBase,
}

#[async_trait(?Send)]
pub trait Keypad {
    async fn wait_key(&self) -> Key;
}
