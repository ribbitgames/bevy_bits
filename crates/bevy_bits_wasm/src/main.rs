#[cfg(target_arch = "wasm32")]
mod main_wasm;

#[cfg(target_arch = "wasm32")]
fn main() -> Result<(), wasm_bindgen::prelude::JsValue> {
    main_wasm::main_wasm()
}

#[cfg(not(target_arch = "wasm32"))]
const fn main() {}
