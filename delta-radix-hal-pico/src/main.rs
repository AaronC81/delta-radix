#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(default_alloc_error_handler)]

use core::panic::PanicInfo;

use alloc_cortex_m::CortexMHeap;
use cortex_m_rt::entry;
use delta_radix_hal::{Hal, Display};
use embedded_hal::digital::v2::OutputPin;
use hal::PicoHal;
use hd44780_driver::{HD44780, Cursor};
use rp_pico::{hal::{Timer, Watchdog, Sio, clocks::init_clocks_and_plls, Clock}, pac, Pins};
use embedded_time::{fixed_point::FixedPoint, rate::Extensions};

extern crate alloc;

mod hal;

fn lives_forever<T: ?Sized>(t: &mut T) -> &'static mut T {
    unsafe { (t as *mut T).as_mut().unwrap() }
}

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();
const HEAP_SIZE: usize = 240_000;

#[entry]
fn main() -> ! {
    // Set up allocator
    {
        use core::mem::MaybeUninit;
        static mut HEAP: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe {
            ALLOCATOR.init(HEAP.as_ptr() as usize, HEAP_SIZE)
        }
    }

    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let mut sio = Sio::new(pac.SIO);

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
        .ok()
        .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().integer());
    let timer = Timer::new(pac.TIMER, &mut pac.RESETS);

    let pins = Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    let mut led = pins.led.into_push_pull_output();

    // const int rs = 11, en = 10, d4 = 9, d5 = 8, d6 = 7, d7 = 6;
    let rs = pins.gpio11.into_push_pull_output();
    let en = pins.gpio10.into_push_pull_output();
    let d4 = pins.gpio9.into_push_pull_output();
    let d5 = pins.gpio8.into_push_pull_output();
    let d6 = pins.gpio7.into_push_pull_output();
    let d7 = pins.gpio6.into_push_pull_output();
    
    let mut lcd = HD44780::new_4bit(rs, en, d4, d5, d6, d7, &mut delay).unwrap();

    let mut hal = PicoHal {
        display: hal::LcdDisplay { lcd, delay: lives_forever(&mut delay) },
        keypad: hal::ButtonMatrix,
        time: hal::DelayTime { delay: lives_forever(&mut delay) },
    };

    let rt = nostd_async::Runtime::new();
    nostd_async::Task::new(delta_radix_os::main(hal)).spawn(&rt).join();
    
    loop {
        led.set_high().unwrap();
        delay.delay_ms(1000);
        led.set_low().unwrap();
        delay.delay_ms(1000);
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}
