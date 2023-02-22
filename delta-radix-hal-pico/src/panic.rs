// We never build for anything other than the Pico, but the `cfg`s make Rust Analyzer shut up about
// redefining a the `panic_handler` lang item

#[cfg(not(any(unix, windows)))]
use core::panic::PanicInfo;

use alloc::{format, vec::Vec, string::String};
use delta_radix_hal::Display;

use crate::hal::{LcdDisplay, PicoHal};

static mut PANIC_HAL: Option<&'static mut PicoHal> = None;

pub fn init_panic_peripherals(hal: &'static mut PicoHal) {
    unsafe {
        PANIC_HAL = Some(hal)
    }
}

struct PanicPeripherals<'d> {
    display: &'d mut LcdDisplay<'d>,
}

#[panic_handler]
#[cfg(not(any(unix, windows)))]
fn panic(info: &PanicInfo) -> ! {
    use crate::hal::enter_bootloader;

    let mut periphs = unsafe { PANIC_HAL.as_mut() }.unwrap();
    periphs.display.clear();
    
    let message = format!("{}", info);
    let chars = message.chars().collect::<Vec<_>>();
    for (i, l) in chars.chunks(20).enumerate() {
        periphs.display.set_position(0, i as u8);
        let line = l.iter().copied().collect::<String>();
        periphs.display.print_string(&line);
    }

    unsafe { enter_bootloader(); }
    loop {}
}
