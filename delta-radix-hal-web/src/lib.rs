#![feature(async_fn_in_trait)]

use wasm_bindgen::prelude::wasm_bindgen;

mod hal;

#[wasm_bindgen]
pub async fn start_os() {
    let mut hal = hal::WebHal::new();
    delta_radix_os::main(&mut hal).await
}
