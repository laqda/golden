pub mod clock;
pub mod game;
pub mod grid;
pub mod lexicon;
pub mod log;

use console_error_panic_hook;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn init() {
    debug!("init wasm");
    console_error_panic_hook::set_once();
}
