#![no_std]
#![cfg_attr(
    all(target_arch = "wasm32", target_feature = "atomics"),
    feature(stdarch_wasm_atomic_wait)
)]

extern crate alloc;
extern crate core;
#[cfg(any(test, not(target_arch = "wasm32")))]
extern crate std;

use crate::{
    arena::{ArenaError, PanicError},
    event::EventError,
    js_client::wire_error,
};

pub mod app;
pub mod arena;
pub mod event;
pub mod field;
pub mod handler;
pub mod js_client;

wire_error! {
    Error {
        Arena(ArenaError) = 1,
        Event(EventError) = 2,
        Panic(PanicError) = 3,
    }
}

#[cfg(target_arch = "wasm32")]
use talc::{sync::TalcLock, wasm::*};

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOCATOR: TalcLock<spinning_top::RawSpinlock, WasmGrowAndClaim, WasmBinning> =
    TalcLock::new(WasmGrowAndClaim);

#[cfg(all(target_arch = "wasm32", not(test)))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    use alloc::format;

    use crate::arena::{PanicError, report_error};

    let location = match info.location() {
        Some(location) => format!("{}:{}", location.file(), location.line()),
        None => alloc::string::String::from("unknown"),
    };

    report_error(Error::Panic(PanicError { location, message: format!("{}", info.message()) }));

    core::arch::wasm32::unreachable()
}

#[cfg(test)]
mod error_tests {
    use alloc::{format, string::String, vec::Vec};

    use super::*;
    use crate::js_client::WireError;

    fn identifiers(error: &Error) -> Vec<u16> {
        let mut path = Vec::new();
        error.identifiers(&mut path);
        path
    }

    #[test]
    fn identifiers_are_the_composed_variant_then_the_module_variant() {
        assert_eq!(identifiers(&Error::Arena(ArenaError::CommandOverflow)), [1, 1]);
        assert_eq!(identifiers(&Error::Event(EventError::Decode)), [2, 1]);
        let panic = PanicError { location: String::new(), message: String::new() };
        assert_eq!(identifiers(&Error::Panic(panic)), [3, 1]);
    }

    #[test]
    fn detail_and_seriousness_come_from_the_module() {
        let panic = Error::Panic(PanicError {
            location: String::from("a.rs:1"),
            message:  String::from("boom"),
        });
        assert_eq!(panic.detail(), "a.rs:1: boom");
        assert!(panic.is_serious());
        assert!(!Error::Event(EventError::Decode).is_serious());
    }

    #[test]
    fn display_is_the_debug_representation() {
        let error = Error::Event(EventError::Decode);
        assert_eq!(format!("{error}"), format!("{error:?}"));
        assert_eq!(format!("{error}"), "Event(Decode)");
    }
}
