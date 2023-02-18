use std::{panic::catch_unwind, cell::RefCell, rc::Rc};

use delta_radix_hal::{Key, Hal};
use delta_radix_os::main;
use futures::executor::block_on;
use hal::TestHal;
use keys::{SetFormat, Number};
use panic_message::panic_message;

use crate::hal::run_os;

mod hal;

#[macro_use]
mod keys;

#[test]
fn test_calc_addition() {
    let hal = run_os(&keys!(
        Number(2),
        Key::Add,
        Number(2),
        Key::Exe,
    ));
    assert_eq!(hal.expression(), "2+2");
    assert_eq!(hal.result(), "4");
    assert!(!hal.overflow());
}

#[test]
fn test_overflow() {
    let hal = run_os(&keys!(
        SetFormat(8, false),
        Number(255),
        Key::Add,
        Number(1),
        Key::Exe,
    ));
    assert_eq!(hal.format(), "U8");
    assert_eq!(hal.expression(), "255+1");
    assert_eq!(hal.result(), "0");
    assert!(hal.overflow());
}
