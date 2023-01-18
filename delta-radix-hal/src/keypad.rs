use async_trait::async_trait;
use alloc::boxed::Box;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Key {
    Digit(u8),
    Shift,
    Menu,
}

#[async_trait]
pub trait Keypad {
    async fn wait_key(&self) -> Key;
    fn poll_key(&self) -> Option<Key>;
}
