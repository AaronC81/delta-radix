#![no_std]
extern crate alloc;

use delta_radix_hal::{Hal, Keypad, Display};

pub async fn main(mut hal: impl Hal) {
    // TODO
    loop {
        hal.keypad().wait_key().await;
        hal.display_mut().print_char('A');
    }
}
