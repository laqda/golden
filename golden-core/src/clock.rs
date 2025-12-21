use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub(crate) struct Clock {
    max: u32,
    remaining_ms: u32,
}

/// The game clock.
#[wasm_bindgen]
impl Clock {
    #[wasm_bindgen(constructor)]
    pub fn new(ms: u32) -> Clock {
        Clock {
            max: ms,
            remaining_ms: ms,
        }
    }

    pub fn substract(&mut self, amount: u32) {
        self.remaining_ms = self.remaining_ms.saturating_sub(amount);
    }

    pub fn remaining_ms(&self) -> u32 {
        self.remaining_ms
    }

    pub fn reset(&mut self) {
        self.remaining_ms = self.max;
    }
}
