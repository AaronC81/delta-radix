#![no_std]
#![feature(let_chains)]

extern crate alloc;

pub mod calc;
pub mod menu;

use calc::CalculatorApplication;
use delta_radix_hal::{Hal, Keypad, Display};

pub async fn main(mut hal: impl Hal) {
    let (disp, keys, _) = hal.common_mut();
    disp.init();

    let mut calc_app = CalculatorApplication::new(&mut hal);
    calc_app.main().await;
}
