//! Module providing a wrapper for the native Julia symbol.

use std::convert::TryFrom;
use std::ffi::CStr;

use super::JlValue;
use crate::error::{Error, Result};
use crate::string::IntoCString;
use crate::{jlvalues, sys::*};

/// Trait implemented by every type which can be used to construct a Symbol.
pub trait IntoSymbol {
    fn into_symbol(self) -> Result<Symbol>;
}

jlvalues! {
    pub struct Symbol(jl_sym_t);
}

impl Symbol {
    /// Construct a new symbol with a name.
    pub fn with_name<S: IntoCString>(name: S) -> Result<Self> {
        let name = name.into_cstring();
        let raw = unsafe { jl_symbol(name.as_ptr()) };
        Self::new(raw).map_err(|_| Error::InvalidSymbol)
    }

    // This never fails.
    /// Procedurally generates a new symbol.
    pub fn gensym() -> Self {
        unsafe {
            let raw = jl_gensym();
            Self::new_unchecked(raw)
        }
    }

    // This never fails.
    /// Returns `symtab`, the root symbol.
    pub fn get_root() -> Self {
        unsafe {
            let raw = jl_get_root_symbol();
            Self::new_unchecked(raw)
        }
    }
}

impl IntoSymbol for Symbol {
    fn into_symbol(self) -> Result<Symbol> {
        Ok(self)
    }
}

impl<S: IntoCString> IntoSymbol for S {
    fn into_symbol(self) -> Result<Symbol> {
        Symbol::with_name(self.into_cstring())
    }
}

impl<'a> TryFrom<&'a Symbol> for String {
    type Error = Error;
    fn try_from(sym: &Symbol) -> Result<Self> {
        let raw = unsafe { jl_symbol_name(sym.lock()?) };
        jl_catch!();
        let cstr = unsafe { CStr::from_ptr(raw as *const std::ffi::c_char) };
        let cstring = cstr.to_owned();
        cstring.into_string().map_err(From::from)
    }
}
