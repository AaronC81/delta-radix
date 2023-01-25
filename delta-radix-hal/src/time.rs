use core::time::Duration;

use async_trait::async_trait;
use alloc::boxed::Box;

#[async_trait(?Send)]
pub trait Time {
    async fn sleep(&mut self, dur: Duration);
}