#![no_std]
extern crate alloc;

mod display;
pub use display::*;

mod keypad;
pub use keypad::*;

pub trait Hal {
    type D: Display;
    type K: Keypad;

    fn display(&self) -> &Self::D;
    fn display_mut(&mut self) -> &mut Self::D;

    fn keypad(&self) -> &Self::K;
    fn keypad_mut(&mut self) -> &mut Self::K;
}
