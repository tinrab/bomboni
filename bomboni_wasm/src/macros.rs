use wasm_bindgen::prelude::*;

/// External JavaScript console functions.
#[wasm_bindgen]
extern "C" {
    /// Logs a message to the browser console.
    #[wasm_bindgen(js_namespace = console)]
    #[allow(unsafe_code)]
    pub fn log(s: &str);

    /// Logs an error message to the browser console.
    #[wasm_bindgen(js_namespace = console)]
    #[allow(unsafe_code)]
    pub fn error(s: &str);
}

/// Logs a formatted message to the browser console.
#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => ($crate::macros::log(&format_args!($($t)*).to_string()))
}

/// Logs a formatted error message to the browser console.
#[macro_export]
macro_rules! console_error {
    ($($t:tt)*) => (error(&format_args!($($t)*).to_string()))
}
