extern crate alloc;
extern crate core;
extern crate std;

pub mod js_client;
pub mod event;
pub mod app;

#[cfg(all(target_family = "wasm", not(target_feature = "atomics")))]
use talc::wasm::{WasmDynamicTalc, new_wasm_dynamic_allocator};

#[cfg(all(target_family = "wasm", not(target_feature = "atomics")))]
#[global_allocator]
static ALLOCATOR: WasmDynamicTalc = new_wasm_dynamic_allocator();

macro_rules! debug_log {
    ($($arg:tt)*) => {{
        web_sys::console::log_1(
            &wasm_bindgen::JsValue::from_str(&format!($($arg)*))
        );
    }};
}
pub(crate) use debug_log;
