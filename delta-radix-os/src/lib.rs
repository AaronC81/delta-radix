#![no_std]
extern crate alloc;

use delta_radix_hal::{Hal, Keypad, Display};

pub async fn main(mut hal: impl Hal) {
    let (disp, keys, _) = hal.common_mut();
    disp.set_position(4, 1);
    disp.print_string("DELTA RADIX");
    disp.set_position(4, 3);
    disp.print_string("Press a key");
    keys.wait_key().await;

    disp.clear();
    disp.set_position(0, 0);
    disp.print_string("U32 ============ 50%");

    loop {
        keys.wait_key().await;
    }
}
