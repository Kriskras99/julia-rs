use std::ffi::CStr;

use julia_sys::{
    jl_atexit_hook, jl_box_float64, jl_call1, jl_eval_string, jl_exception_occurred,
    jl_float64_type, jl_init, jl_is_initialized, jl_typeis, jl_unbox_float64, jl_value_t,
};

unsafe fn eval(string: &str) -> *mut jl_value_t {
    let bytes = string.as_bytes();
    let string = CStr::from_bytes_with_nul(bytes).unwrap();
    let result = jl_eval_string(string.as_ptr());
    assert!(jl_exception_occurred().is_null());

    result
}

fn main() {
    unsafe {
        jl_init();
        assert!(jl_is_initialized() != 0);

        eval("f(x) = x * 2 - 1\0");
        let f = eval("f\0");

        let x = jl_box_float64(3.0);

        let ret = jl_call1(f, x);

        let y = if jl_typeis(ret, jl_float64_type) {
            jl_unbox_float64(ret)
        } else {
            panic!("f is not a Float64")
        };

        assert_eq!(y, 3.0 * 2.0 - 1.0);
        println!("f({}) = {}", 3.0, y);

        jl_atexit_hook(0);
    }
}
