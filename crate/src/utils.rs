use std::process;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        #[cfg(feature = "console_error_panic_hook")]
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

impl<T> WasmUnwrap<T> for Option<T> {
    fn unwrap_wasm(self) -> T {
        match self {
            Some(w) => w,
            None => process::abort(),
        }
    }
}

impl<T, E> WasmUnwrap<T> for Result<T, E> {
    fn unwrap_wasm(self) -> T {
        match self {
            Ok(w) => w,
            Err(_e) => process::abort(),
        }
    }
}

pub trait WasmUnwrap<T> {
    fn unwrap_wasm(self) -> T;
}