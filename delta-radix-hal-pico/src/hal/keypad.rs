use core::convert::Infallible;

use alloc::boxed::Box;
use async_trait::async_trait;
use cortex_m::delay::Delay;
use delta_radix_hal::Key;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use rp_pico::hal::gpio::{bank0::{Gpio15, Gpio16, Gpio17, Gpio18, Gpio19, Gpio20, Gpio21, Gpio22, Gpio26, Gpio27, Gpio28}, Pin, Input, PullUp, Output, PushPull};

type Col0 = Gpio15;
type Col1 = Gpio16;
type Col2 = Gpio17;
type Col3 = Gpio18;
type Col4 = Gpio19;

type Row0 = Gpio20;
type Row1 = Gpio21;
type Row2 = Gpio22;
type Row3 = Gpio26;
type Row4 = Gpio27;
type Row5 = Gpio28;

type ColPin<T> = Pin<T, Input<PullUp>>;
type RowPin<T> = Pin<T, Output<PushPull>>;

pub struct ButtonMatrix<'d> {
    pub delay: &'d mut Delay,

    pub col0: ColPin<Col0>,
    pub col1: ColPin<Col1>,
    pub col2: ColPin<Col2>,
    pub col3: ColPin<Col3>,
    pub col4: ColPin<Col4>,

    pub row0: RowPin<Row0>,
    pub row1: RowPin<Row1>,
    pub row2: RowPin<Row2>,
    pub row3: RowPin<Row3>,
    pub row4: RowPin<Row4>,
    pub row5: RowPin<Row5>,
}

impl<'d> ButtonMatrix<'d> {
    const COLS: usize = 5;
    const ROWS: usize = 6;

    fn rows_and_cols(&mut self) ->
        ([&mut dyn OutputPin<Error = Infallible>; ButtonMatrix::<'d>::ROWS], [&mut dyn InputPin<Error = Infallible>; ButtonMatrix::<'d>::COLS])
    {
        // Borrow splitting FTW!
        (
            [&mut self.row0, &mut self.row1, &mut self.row2, &mut self.row3, &mut self.row4, &mut self.row5],
            [&mut self.col0, &mut self.col1, &mut self.col2, &mut self.col3, &mut self.col4],
        )
    }

    pub fn scan_matrix(&mut self) -> Option<(u8, u8)> {
        let (mut rows, mut cols) = self.rows_and_cols();

        // Set all rows high
        for row in rows.iter_mut() {
            row.set_high().unwrap();
        }

        // Iterate over each row...
        for (r, row) in rows.iter_mut().enumerate() {
            // Set it low
            row.set_low().unwrap();

            // Check each column - if it's low, the button was pressed!
            for (c, col) in cols.iter_mut().enumerate() {
                if col.is_low().unwrap() {
                    return Some((r as u8, c as u8));
                }
            }

            // Put it back to high
            row.set_high().unwrap();
        }

        // Nothing pressed
        None
    }

    pub fn map_key(&self, row: u8, col: u8) -> Option<Key> {
        // TODO: these keycodes are for my breadboard prototype and will need changing
        match (row, col) {
            (0, 0) => Some(Key::Exe),
            
            (4, 0) => Some(Key::Digit(0)),
            (3, 0) => Some(Key::Digit(1)),
            (3, 1) => Some(Key::Digit(2)),
            (3, 2) => Some(Key::Digit(3)),
            (2, 0) => Some(Key::Digit(4)),
            (2, 1) => Some(Key::Digit(5)),
            (2, 2) => Some(Key::Digit(6)),
            (1, 0) => Some(Key::Digit(7)),
            (1, 1) => Some(Key::Digit(8)),
            (1, 2) => Some(Key::Digit(9)),

            (0, 3) => {
                // Handy bootloader button
                unsafe {
                    // Resolve a function which allows us to look up items in ROM tables
                    let rom_table_lookup_fn_addr = *(0x18 as *const u16) as *const ();
                    let rom_table_lookup_fn: extern "C" fn(*const u16, u32) -> *const () = core::mem::transmute(rom_table_lookup_fn_addr);
                    
                    // Use that function to look up the address of the USB bootloader function
                    let usb_boot_fn_code = (('B' as u32) << 8) | ('U' as u32);
                    let func_table = *(0x14 as *const u16) as *const u16;
                    let usb_boot_fn_addr = rom_table_lookup_fn(func_table, usb_boot_fn_code);

                    // Call that function
                    let usb_boot_fn: extern "C" fn(u32, u32) = core::mem::transmute(usb_boot_fn_addr);
                    usb_boot_fn(0, 0);
                }
                panic!("failed to access bootloader")
            }
            _ => None,
        }
    }
}

#[async_trait(?Send)]
impl<'d> delta_radix_hal::Keypad for ButtonMatrix<'d> {
    async fn wait_key(&mut self) -> Key {
        loop {
            if let Some((r, c)) = self.scan_matrix() {
                if let Some(key) = self.map_key(r, c) {
                    return key
                }
            }

            self.delay.delay_ms(5);
        }
    }
}
