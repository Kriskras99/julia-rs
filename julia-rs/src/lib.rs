//! Safe and idiomatic [Julia](https://julialang.org) bindings for
//! [Rust](https://rust-lang.org).
//! [#JuliaLang](https://twitter.com/search?q=%23JuliaLang)
//! [#RustLang](https://twitter.com/search?q=%23RustLang)
//!
//! Uses nightly Rust for compilation, rustfmt with default settings for
//! formatting, clippy for checking and resolving lints.
//!
//! julia-sys are the raw ffi bindings for Julia generated with
//! [bindgen](https://crates.io/crates/bindgen).
//!
//! # Example
//!
//! An example of using Rust to interface with Julia.
//!
//! ```
//! use julia::api::{Julia, Value};
//!
//! let mut jl = Julia::new().unwrap();
//! jl.eval_string("println(\"Hello, Julia!\")").unwrap();
//! // Hello, Julia!
//!
//! let sqrt = jl.base().function("sqrt").unwrap();
//!
//! let boxed_x = Value::from(1337.0);
//! let boxed_sqrt_x = sqrt.call1(&boxed_x).unwrap();
//!
//! let sqrt_x = f64::try_from(boxed_sqrt_x).unwrap();
//! println!("{}", sqrt_x);
//! // 36.565010597564445
//! ```

#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::doc_markdown)]

pub mod error;
pub mod ext;
pub mod string;
pub mod sys;
pub mod version;

pub mod api;

#[cfg(test)]
mod tests {
    use super::api::Julia;

    #[test]
    fn sanity() {
        let _jl = Julia::new();
    }
}
