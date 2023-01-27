use core::time::Duration;
use alloc::boxed::Box;
use async_trait::async_trait;
use cortex_m::delay::Delay;

pub struct DelayTime<'d> {
    pub delay: &'d mut Delay,
}

#[async_trait(?Send)]
impl<'d> delta_radix_hal::Time for DelayTime<'d> {
    async fn sleep(&mut self, dur: Duration) {
        self.delay.delay_ms(dur.as_millis() as u32)
    }
}

