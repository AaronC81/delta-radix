use delta_radix_hal::{Hal, Display};
use hal::SimHal;

mod hal;

#[tokio::main]
async fn main() {
    let mut hal = SimHal::new();
    hal.display_mut().init();

    delta_radix_os::main(&mut hal).await;
}
