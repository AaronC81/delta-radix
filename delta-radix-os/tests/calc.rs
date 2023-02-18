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

#[test]
fn test_hex_input() {
    let hal = run_os(&keys!(
        // Both base as a prefix...
        Key::HexBase,
        Key::Digit(1),
        Key::Digit(0xA),
        Key::Add,
        // ...and a suffix
        Key::Digit(0xC),
        Key::HexBase,
        Key::Exe,
    ));
    assert_eq!(hal.expression(), "x1A+Cx");
    assert_eq!(hal.result(), "38");
    assert!(!hal.overflow());
}

#[test]
fn test_hex_result() {
    let hal = run_os(&keys!(
        Key::FormatSelect,
        Key::HexBase,
        Number(0xA1C),
        Key::Exe,
    ));
    assert_eq!(hal.expression(), 0xA1C.to_string());
    assert_eq!(hal.result(), "xA1C");
    assert!(!hal.overflow());
}

#[test]
fn test_binary_input() {
    let hal = run_os(&keys!(
        // Both base as a prefix...
        Key::BinaryBase,
        Key::Digit(1),
        Key::Digit(1),
        Key::Digit(0),
        Key::Digit(1),
        Key::Add,
        // ...and a suffix
        Key::Digit(1),
        Key::Digit(1),
        Key::Digit(0),
        Key::BinaryBase,
        Key::Exe,
    ));
    assert_eq!(hal.expression(), "b1101+110b");
    assert_eq!(hal.result(), (0b1101 + 0b110).to_string());
    assert!(!hal.overflow());
}

#[test]
fn test_binary_result() {
    let hal = run_os(&keys!(
        Key::FormatSelect,
        Key::BinaryBase,
        Number(0b11011101),
        Key::Exe,
    ));
    assert_eq!(hal.expression(), 0b11011101.to_string());
    assert_eq!(hal.result(), "b11011101");
    assert!(!hal.overflow());
}
