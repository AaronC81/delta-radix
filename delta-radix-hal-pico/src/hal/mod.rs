
pub mod display;
pub mod keypad;
pub mod time;

use async_trait::async_trait;
use alloc::boxed::Box;
use delta_radix_hal::Display;

pub use self::{display::LcdDisplay, keypad::ButtonMatrix, time::DelayTime};

pub struct PicoHal<'d> {
    pub display: LcdDisplay<'d>,
    pub keypad: ButtonMatrix<'d>,
    pub time: DelayTime<'d>,
}

#[async_trait(?Send)]
impl<'d> delta_radix_hal::Hal for PicoHal<'d> {
    type D = LcdDisplay<'d>;
    type K = ButtonMatrix<'d>;
    type T = DelayTime<'d>;

    fn display(&self) -> &Self::D { &self.display }
    fn display_mut(&mut self) -> &mut Self::D { &mut self.display }

    fn keypad(&self) -> &Self::K { &self.keypad }
    fn keypad_mut(&mut self) -> &mut Self::K { &mut self.keypad }

    fn time(&self) -> &Self::T { &self.time }
    fn time_mut(&mut self) -> &mut Self::T { &mut self.time }

    fn common_mut(&mut self) -> (&mut Self::D, &mut Self::K, &mut Self::T) {
        (&mut self.display, &mut self.keypad, &mut self.time)
    }

    async fn enter_bootloader(&mut self) {
        let display = self.display_mut();
        display.clear();
        display.set_position(4, 1);
        display.print_string("Bootloader!");

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
}
