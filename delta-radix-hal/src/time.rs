use core::time::Duration;

use async_trait::async_trait;
use alloc::boxed::Box;

pub trait Time {
    async fn sleep(&mut self, dur: Duration);
}