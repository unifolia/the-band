pub mod euclid;
pub mod instruments;
pub mod music;
pub mod percussion;
pub mod render;
pub mod rng;
pub mod sequence;
pub mod synth;
pub mod utils;

pub use sequence::generate_band;

#[cfg(target_arch = "wasm32")]
use js_sys::Float32Array;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
fn seed_from_parts(seed_hi: u32, seed_lo: u32) -> u64 {
    ((seed_hi as u64) << 32) | seed_lo as u64
}

#[cfg(target_arch = "wasm32")]
fn js_error(message: impl Into<String>) -> JsValue {
    JsValue::from_str(&message.into())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn generate_band_json(seed_hi: u32, seed_lo: u32) -> Result<String, JsValue> {
    let band = sequence::generate_band(seed_from_parts(seed_hi, seed_lo));
    serde_json::to_string(&band).map_err(|error| js_error(error.to_string()))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn render_loop(seed_hi: u32, seed_lo: u32, sample_rate: f32) -> Result<Float32Array, JsValue> {
    let output = render::render_loop(seed_from_parts(seed_hi, seed_lo), sample_rate)
        .map_err(js_error)?;
    Ok(Float32Array::from(output.as_slice()))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct Engine {
    inner: render::RenderEngine,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl Engine {
    #[wasm_bindgen(constructor)]
    pub fn new(seed_hi: u32, seed_lo: u32, sample_rate: f32) -> Result<Engine, JsValue> {
        let inner = render::RenderEngine::from_seed(seed_from_parts(seed_hi, seed_lo), sample_rate)
            .map_err(js_error)?;
        Ok(Engine { inner })
    }

    pub fn reset(&mut self) {
        self.inner.reset();
    }

    pub fn render(&mut self, frames: usize) -> Result<(), JsValue> {
        self.inner.render(frames).map_err(js_error)
    }

    pub fn output_ptr(&self) -> usize {
        self.inner.output_ptr() as usize
    }

    pub fn output_len(&self) -> usize {
        self.inner.output_len()
    }

    pub fn output_view(&self) -> Float32Array {
        unsafe { Float32Array::view(self.inner.output()) }
    }

    pub fn loop_samples(&self) -> usize {
        self.inner.loop_samples()
    }

    pub fn debug_summary_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(self.inner.band()).map_err(|error| js_error(error.to_string()))
    }
}
