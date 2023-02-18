use std::{panic::catch_unwind, cell::RefCell, rc::Rc};

use delta_radix_hal::{Key, Hal};
use delta_radix_os::main;
use futures::executor::block_on;
use hal::TestHal;
use panic_message::panic_message;

use crate::hal::run_os;

mod hal;

#[test]
fn test_calc_addition() {
    let hal = run_os(&[
        Key::Digit(2),
        Key::Add,
        Key::Digit(2),
        Key::Exe,
    ]);
    assert_eq!(hal.expression(), "2+2");
    assert_eq!(hal.result(), "4");
}