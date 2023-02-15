use wasm_bindgen::prelude::wasm_bindgen;

mod hal;

#[wasm_bindgen]
pub async fn main() {
    let hal = hal::WebHal::new();
    delta_radix_os::main(hal).await
}
