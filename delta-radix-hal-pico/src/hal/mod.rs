
pub mod display;
pub mod keypad;
pub mod time;
pub use self::{display::LcdDisplay, keypad::ButtonMatrix, time::DelayTime};

pub struct PicoHal<'d> {
    pub display: LcdDisplay<'d>,
    pub keypad: ButtonMatrix<'d>,
    pub time: DelayTime<'d>,
}

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
}
