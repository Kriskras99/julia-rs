use std::ffi::CStr;

use julia_sys::{
    jl_atexit_hook, jl_eval_string, jl_exception_occurred, jl_init, jl_is_initialized,
};

fn main() {
    unsafe {
        jl_init();
        assert!(jl_is_initialized() != 0);

        let bytes = b"print(\"Hello, world!\")\0";
        let string = CStr::from_bytes_with_nul(bytes).unwrap();
        jl_eval_string(string.as_ptr());
        assert!(jl_exception_occurred().is_null());

        jl_atexit_hook(0);
    }
}
