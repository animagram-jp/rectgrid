extern crate alloc;
extern crate core;
extern crate std;

pub mod js_client;
pub mod event;
pub mod app;

// ============================================================
// Global Allocator
// ============================================================

// app repositoryと同じtalcを使う。examplesはシングルスレッド
// (SharedArrayBuffer/atomics不使用)なので、ロックを要さないwasm向けの
// Cellベース実装 (WasmDynamicTalc) で足りる。app repositoryが
// worker+共有メモリ用に使うTalcLock<spinning_top::RawSpinlock, ...>は
// ここでは不要。
#[cfg(all(target_family = "wasm", not(target_feature = "atomics")))]
use talc::wasm::{WasmDynamicTalc, new_wasm_dynamic_allocator};

#[cfg(all(target_family = "wasm", not(target_feature = "atomics")))]
#[global_allocator]
static ALLOCATOR: WasmDynamicTalc = new_wasm_dynamic_allocator();

// ============================================================
// log
// ============================================================

macro_rules! debug_log {
    ($($arg:tt)*) => {{
        web_sys::console::log_1(
            &wasm_bindgen::JsValue::from_str(&format!($($arg)*))
        );
    }};
}
pub(crate) use debug_log;

// ============================================================
// no_std
// ============================================================

// #![no_std]
// use core::{
//     panic::Panicinfo,
//     arch::wasm32::unreachable
// };
//
// #[panic_handler]
// fn panic(info: &PanicInfo) -> ! {
//     debug_log!("panic: {}", info);
//     unreachable()
// }
