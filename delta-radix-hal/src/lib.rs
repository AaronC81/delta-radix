#![no_std]
#![feature(async_fn_in_trait)]

extern crate alloc;
use async_trait::async_trait;
use alloc::boxed::Box;

mod display;
pub use display::*;

mod keypad;
pub use keypad::*;

mod time;
pub use time::*;

pub trait Hal {
    type D: Display;
    type K: Keypad;
    type T: Time;

    fn display(&self) -> &Self::D;
    fn display_mut(&mut self) -> &mut Self::D;

    fn keypad(&self) -> &Self::K;
    fn keypad_mut(&mut self) -> &mut Self::K;

    fn time(&self) -> &Self::T;
    fn time_mut(&mut self) -> &mut Self::T;

    fn common_mut(&mut self) -> (&mut Self::D, &mut Self::K, &mut Self::T);

    async fn enter_bootloader(&mut self);
}
