pub mod date_time;
pub mod id;
pub mod macros;

#[doc(hidden)]
#[cfg(all(
    target_family = "wasm",
    not(any(target_os = "emscripten", target_os = "wasi")),
    feature = "wasm"
))]
pub mod bomboni {
    pub mod wasm {
        pub use bomboni_wasm::*;
    }
}
