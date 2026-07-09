extern crate core;
extern crate alloc;
extern crate std;

pub mod js_client;
pub mod event;
pub mod app;

// ============================================================
// Global Allocator
// ============================================================

#[cfg(target_arch = "wasm32")]
use dlmalloc::GlobalDlmalloc;

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOCATOR: GlobalDlmalloc = GlobalDlmalloc;

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