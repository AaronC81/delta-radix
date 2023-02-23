use core::time::Duration;

pub trait Time {
    async fn sleep(&mut self, dur: Duration);
}