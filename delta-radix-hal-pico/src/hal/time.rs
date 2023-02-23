use core::time::Duration;
use alloc::boxed::Box;
use async_trait::async_trait;
use cortex_m::delay::Delay;

pub struct DelayTime<'d> {
    pub delay: &'d mut Delay,
}

impl<'d> delta_radix_hal::Time for DelayTime<'d> {
    async fn sleep(&mut self, dur: Duration) {
        self.delay.delay_us(dur.as_micros() as u32)
    }
}

