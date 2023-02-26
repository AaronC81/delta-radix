#![no_std]
#![feature(let_chains)]
#![feature(async_fn_in_trait)]

extern crate alloc;

pub mod calc;

use calc::frontend::CalculatorApplication;
use delta_radix_hal::{Hal, Display};

pub async fn main(hal: &mut impl Hal) {
    let (disp, _, _) = hal.common_mut();
    disp.init();

    let mut calc_app = CalculatorApplication::new(hal);
    calc_app.main().await;
}
