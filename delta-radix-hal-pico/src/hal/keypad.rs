use alloc::boxed::Box;
use async_trait::async_trait;
use delta_radix_hal::Key;

pub struct ButtonMatrix;

#[async_trait(?Send)]
impl delta_radix_hal::Keypad for ButtonMatrix {
    async fn wait_key(&self) -> Key {
        // TODO
        loop {}
    }
}
