#![allow(
    clippy::allow_attributes,
    reason = "allow attributes are needed for wasm"
)]
// This crate is meant to run a single bit

use core::str::FromStr;

use ribbit_bits::BitName;
use wasm_bindgen::prelude::*;
use web_sys::console;

// run function generation

include!(concat!(env!("OUT_DIR"), "/bit_runner_impl.rs"));

struct BitRunner {
    pub bit_name: BitName,
}

impl FromStr for BitRunner {
    type Err = ::strum::ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            bit_name: BitName::from_str(s)?,
        })
    }
}

/// Defined in javasccript

#[wasm_bindgen(module = "/communication.js")]
extern "C" {
    // Since we can't pass parameters to main and that bevy needs to be init in the main,
    // we gather the bit_name from a javascript call.
    #[allow(unsafe_code, reason = "unsafe code is needed for wasm")]
    fn get_bit_name() -> String;
}

pub(crate) fn main_wasm() -> Result<(), JsValue> {
    let bit_name = get_bit_name();
    let Ok(bit_runner) = BitRunner::from_str(&bit_name) else {
        return Err(JsValue::from_str(&format!("Invalid BitName: {bit_name}")));
    };
    console::log_1(&format!("Starting {}", bit_runner.bit_name).into());
    bit_runner.run();
    Ok(())
}
