#![no_std]
#![feature(let_chains)]

extern crate alloc;

pub mod calc;

use calc::CalculatorApplication;
use delta_radix_hal::{Hal, Keypad, Display};

pub async fn main(mut hal: impl Hal) {
    let (disp, keys, _) = hal.common_mut();
    disp.set_position(4, 1);
    disp.print_string("DELTA RADIX");
    disp.set_position(4, 3);
    disp.print_string("Press a key");
    keys.wait_key().await;

    let mut calc_app = CalculatorApplication::new(&mut hal);
    calc_app.main().await;
}
