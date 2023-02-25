use delta_radix_hal::{Key, Keypad};
use embedded_time::duration::{Extensions, Duration, Seconds, Microseconds};
use rp_pico::{pac::{self, interrupt}, hal::{Sio, multicore::Stack, sio::SioFifo, timer::Alarm0, Timer}, Pins};

use crate::{lives_forever, executor};

use super::ButtonMatrix;

pub struct AsyncKeypadReceiver<'s> {
    pub fifo: &'s mut SioFifo,
}

impl<'s> delta_radix_hal::Keypad for AsyncKeypadReceiver<'s> {
    async fn wait_key(&mut self) -> Key {
        loop {
            if let Some(key) = Key::from_u32(self.fifo.read_blocking()) {
                return key;
            }
        }
    }
}

pub const ASYNC_KEYPAD_START_MAGIC: u32 = 0xCAFECAFE;

pub fn async_keypad_core1() -> ! {
    // Grab some important peripherals
    let mut pac = unsafe { pac::Peripherals::steal() };
    let core = unsafe { pac::CorePeripherals::steal() };
    let mut sio = Sio::new(pac.SIO);
    let mut delay = cortex_m::delay::Delay::new(core.SYST, 125_000_000);
    let pins = Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Wait until the magic word over FIFO
    loop {
        if sio.fifo.read_blocking() == ASYNC_KEYPAD_START_MAGIC {
            break;
        }
    }

    // Set up button matrix
    let mut matrix = ButtonMatrix {
        delay: lives_forever(&mut delay),

        col0: pins.gpio15.into_pull_up_input(),
        col1: pins.gpio16.into_pull_up_input(),
        col2: pins.gpio17.into_pull_up_input(),
        col3: pins.gpio18.into_pull_up_input(),
        col4: pins.gpio19.into_pull_up_input(),

        row0: pins.gpio20.into_push_pull_output(),
        row1: pins.gpio21.into_push_pull_output(),
        row2: pins.gpio22.into_push_pull_output(),
        row3: pins.gpio26.into_push_pull_output(),
        row4: pins.gpio27.into_push_pull_output(),
        row5: pins.gpio28.into_push_pull_output(),

        currently_pressed: None,
    };

    // Set up timer stuff
    unsafe { pac::NVIC::unmask(pac::Interrupt::TIMER_IRQ_0); }
    let mut timer = Timer::new(pac.TIMER, &mut pac.RESETS);
    let mut alarm = timer.alarm_0().unwrap();
    alarm.schedule(Microseconds(5_000_000)).unwrap();
    alarm.enable_interrupt();
    
    // For the rest of time, loop looking for buttons
    loop {
        let key = executor::execute(matrix.wait_key());
        sio.fifo.write(key.to_u32());
    }
}

#[interrupt]
fn TIMER_IRQ_0() {
    let mut pac = unsafe { pac::Peripherals::steal() };
    let mut sio = Sio::new(pac.SIO);

    sio.fifo.write(Key::Digit(1).to_u32());
    sio.fifo.write(Key::Digit(2).to_u32());
    sio.fifo.write(Key::Digit(3).to_u32());

    let mut timer = Timer::new(pac.TIMER, &mut pac.RESETS);
    let mut alarm = timer.alarm_0().unwrap();
    alarm.clear_interrupt();
}
