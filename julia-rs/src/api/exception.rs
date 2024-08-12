//! Module providing wrappers for the native Julia exceptions.

use std::fmt;
use std::ops::Deref;
use std::ops::DerefMut;

use smallvec::SmallVec;

use super::{Datatype, JlValue, Symbol, Value};
use crate::error::Result;
use crate::string::IntoCString;
use crate::sys::*;

/// Enum containing different Julia exceptions wrapped as a Value.
#[derive(Debug, Clone)]
pub enum Exception {
    /// The parameters to a function call do not match a valid signature
    Argument(Value),
    /// Attempt to access index out-of-bounds
    Bounds(Value),
    /// Composite exception
    Composite(Value),
    /// Divide by zero
    Divide(Value),
    /// The argument is outside of the valid domain
    Domain(Value),
    /// No more data is available from file or stream
    EOF(Value),
    /// Generic error occurred
    Error(Value),
    /// Type conversion cannot be done exactly
    Inexact(Value),
    /// An error occurred when running a module's __init__
    Init(Value),
    /// The process was stopped by a terminal interrupt (^C)
    Interrupt(Value),
    /// The program reached an invalid exception
    InvalidState(Value),
    /// Key doesn't exist in Associative- or Set-like object
    Key(Value),
    /// An error occurred while include-ing, require-ing or using a file
    Load(Value),
    /// Operation allocated too much memory
    OutOfMemory(Value),
    /// Operation tried to write to read-only memory
    ReadOnlyMemory(Value),
    /// Remote exception occurred
    Remote(Value),
    /// Method with the required type signature doesn't exist
    Method(Value),
    /// The result of an expression is too large
    Overflow(Value),
    /// The expression couldn't be parsed as a valid Julia expression
    Parse(Value),
    /// System call failed
    System(Value),
    /// Type assertion failed
    Type(Value),
    /// The item or field is not defined
    UndefRef(Value),
    /// Symbol is not defined in current scope
    UndefVar(Value),
    /// Byte array does not represent a valid unicode string
    Unicode(Value),
    /// Unknown exception
    Unknown(Value),
}

impl Exception {
    /// Check if an exception occurred without checking its value.
    pub fn occurred() -> bool {
        unsafe { !jl_exception_occurred().is_null() }
    }

    /// Catch an exception if it occurred. Returns None if no exception
    /// occurred.
    pub fn catch() -> Option<Self> {
        let raw = unsafe { jl_exception_occurred() };
        unsafe {
            jl_exception_clear();
        }
        Value::new(raw).and_then(Self::with_value).ok()
    }

    // TODO: replace comparing typename with comparing a *mut jl_datatype_t.
    /// Construct a new Exception with a wrapped Julia value.
    pub fn with_value(value: Value) -> Result<Self> {
        let typename = value.typename()?;
        let ex = match typename.as_str() {
            "ArgumentError" => Self::Argument(value),
            "BoundsError" => Self::Bounds(value),
            "CompositeException" => Self::Composite(value),
            "DivideError" => Self::Divide(value),
            "DomainError" => Self::Domain(value),
            "EOFError" => Self::EOF(value),
            "ErrorException" => Self::Error(value),
            "InexactError" => Self::Inexact(value),
            "InitError" => Self::Init(value),
            "InterruptException" => Self::Interrupt(value),
            "InvalidStateException" => Self::InvalidState(value),
            "KeyError" => Self::Key(value),
            "LoadError" => Self::Load(value),
            "OutOfMemoryError" => Self::OutOfMemory(value),
            "ReadOnlyMemoryError" => Self::ReadOnlyMemory(value),
            "RemoteException" => Self::Remote(value),
            "MethodError" => Self::Method(value),
            "OverflowError" => Self::Overflow(value),
            "ParseError" => Self::Parse(value),
            "SystemError" => Self::System(value),
            "TypeError" => Self::Type(value),
            "UndefRefError" => Self::UndefRef(value),
            "UndefVarError" => Self::UndefVar(value),
            "UnicodeError" => Self::Unicode(value),
            _ => Self::Unknown(value),
        };
        Ok(ex)
    }

    /// Immutably borrows the inner value.
    pub const fn inner_ref(&self) -> &Value {
        match *self {
            Self::Argument(ref value) => value,
            Self::Bounds(ref value) => value,
            Self::Composite(ref value) => value,
            Self::Divide(ref value) => value,
            Self::Domain(ref value) => value,
            Self::EOF(ref value) => value,
            Self::Error(ref value) => value,
            Self::Inexact(ref value) => value,
            Self::Init(ref value) => value,
            Self::Interrupt(ref value) => value,
            Self::InvalidState(ref value) => value,
            Self::Key(ref value) => value,
            Self::Load(ref value) => value,
            Self::OutOfMemory(ref value) => value,
            Self::ReadOnlyMemory(ref value) => value,
            Self::Remote(ref value) => value,
            Self::Method(ref value) => value,
            Self::Overflow(ref value) => value,
            Self::Parse(ref value) => value,
            Self::System(ref value) => value,
            Self::Type(ref value) => value,
            Self::UndefRef(ref value) => value,
            Self::UndefVar(ref value) => value,
            Self::Unicode(ref value) => value,
            Self::Unknown(ref value) => value,
        }
    }

    /// Mutably borrows the inner value.
    pub fn inner_mut(&mut self) -> &mut Value {
        match *self {
            Self::Argument(ref mut value) => value,
            Self::Bounds(ref mut value) => value,
            Self::Composite(ref mut value) => value,
            Self::Divide(ref mut value) => value,
            Self::Domain(ref mut value) => value,
            Self::EOF(ref mut value) => value,
            Self::Error(ref mut value) => value,
            Self::Inexact(ref mut value) => value,
            Self::Init(ref mut value) => value,
            Self::Interrupt(ref mut value) => value,
            Self::InvalidState(ref mut value) => value,
            Self::Key(ref mut value) => value,
            Self::Load(ref mut value) => value,
            Self::OutOfMemory(ref mut value) => value,
            Self::ReadOnlyMemory(ref mut value) => value,
            Self::Remote(ref mut value) => value,
            Self::Method(ref mut value) => value,
            Self::Overflow(ref mut value) => value,
            Self::Parse(ref mut value) => value,
            Self::System(ref mut value) => value,
            Self::Type(ref mut value) => value,
            Self::UndefRef(ref mut value) => value,
            Self::UndefVar(ref mut value) => value,
            Self::Unicode(ref mut value) => value,
            Self::Unknown(ref mut value) => value,
        }
    }

    /// Consumes self and returns the inner value.
    pub fn into_inner(self) -> Value {
        match self {
            Self::Argument(value) => value,
            Self::Bounds(value) => value,
            Self::Composite(value) => value,
            Self::Divide(value) => value,
            Self::Domain(value) => value,
            Self::EOF(value) => value,
            Self::Error(value) => value,
            Self::Inexact(value) => value,
            Self::Init(value) => value,
            Self::Interrupt(value) => value,
            Self::InvalidState(value) => value,
            Self::Key(value) => value,
            Self::Load(value) => value,
            Self::OutOfMemory(value) => value,
            Self::ReadOnlyMemory(value) => value,
            Self::Remote(value) => value,
            Self::Method(value) => value,
            Self::Overflow(value) => value,
            Self::Parse(value) => value,
            Self::System(value) => value,
            Self::Type(value) => value,
            Self::UndefRef(value) => value,
            Self::UndefVar(value) => value,
            Self::Unicode(value) => value,
            Self::Unknown(value) => value,
        }
    }
}

impl Deref for Exception {
    type Target = Value;
    fn deref(&self) -> &Value {
        self.inner_ref()
    }
}

impl DerefMut for Exception {
    fn deref_mut(&mut self) -> &mut Value {
        self.inner_mut()
    }
}

// TODO
impl fmt::Display for Exception {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let description = match *self {
            Self::Argument(_) => "the parameters to a function call do not match a valid signature",
            Self::Bounds(_) => "attempt to access index out-of-bounds",
            Self::Composite(_) => "composite exception",
            Self::Divide(_) => "divide by zero",
            Self::Domain(_) => "the argument is outside of the valid domain",
            Self::EOF(_) => "no more data is available from file or stream",
            Self::Error(_) => "generic error occurred",
            Self::Inexact(_) => "type conversion cannot be done exactly",
            Self::Init(_) => "an error occurred when running a module's __init__ ",
            Self::Interrupt(_) => "the process was stopped by a terminal interrupt (^C)",
            Self::InvalidState(_) => "the program reached an invalid exception",
            Self::Key(_) => "key doesn't exist in Associative- or Set-like object",
            Self::Load(_) => "an error occurred while include-ing, require-ing or using a file",
            Self::OutOfMemory(_) => "operation allocated too much memory",
            Self::ReadOnlyMemory(_) => "operation tried to write to read-only memory",
            Self::Remote(_) => "remote exception occurred",
            Self::Method(_) => "method with the required type signature doesn't exist",
            Self::Overflow(_) => "the result of an expression is too large",
            Self::Parse(_) => "the expression couldn't be parsed as a valid Julia expression",
            Self::System(_) => "system call failed",
            Self::Type(_) => "type assertion failed",
            Self::UndefRef(_) => "the item or field is not defined",
            Self::UndefVar(_) => "symbol is not defined in current scope",
            Self::Unicode(_) => "byte array does not represent a valid unicode string",
            Self::Unknown(_) => "unknown exception",
        };
        f.write_str(description)
    }
}

/// Throws a generic error.
pub fn error<S: IntoCString>(string: S) {
    let string = string.into_cstring();
    let string = string.as_ptr();
    unsafe {
        jl_error(string);
    }
}

/// Throws a formatted generic error.
pub fn error_format(args: fmt::Arguments) {
    error(fmt::format(args).into_cstring());
}

/// Throws an exception with the specified Datatype and message.
pub fn exception<S: IntoCString>(ty: &Datatype, string: S) -> ! {
    let ty = ty.lock().unwrap();
    let string = string.into_cstring();
    let string = string.as_ptr();
    unsafe {
        jl_exceptionf(ty, string);
    }
}

/// Throws an exception with the specified Datatype and a formatted message.
pub fn exception_format(ty: &Datatype, args: fmt::Arguments) -> ! {
    exception(ty, fmt::format(args).into_cstring())
}

/// Too few arguments exception.
pub fn too_few_args<S: IntoCString>(fname: S, min: usize) {
    let fname = fname.into_cstring();
    let fname = fname.as_ptr();
    unsafe {
        jl_too_few_args(fname, min as i32);
    }
}

/// Too many arguments exception.
pub fn too_many_args<S: IntoCString>(fname: S, max: usize) {
    let fname = fname.into_cstring();
    let fname = fname.as_ptr();
    unsafe {
        jl_too_many_args(fname, max as i32);
    }
}

/// Invalid type in an expression.
pub fn type_error<S: IntoCString>(fname: S, expected: &Value, got: &Value) -> ! {
    let fname = fname.into_cstring();
    let fname = fname.as_ptr();
    let expected = expected.lock().unwrap();
    let got = got.lock().unwrap();
    unsafe {
        jl_type_error(fname, expected, got);
    }
}

pub fn type_error_rt<S: IntoCString>(fname: S, context: S, ty: &Value, got: &Value) -> ! {
    let fname = fname.into_cstring();
    let fname = fname.as_ptr();
    let context = context.into_cstring();
    let context = context.as_ptr();
    let ty = ty.lock().unwrap();
    let got = got.lock().unwrap();
    unsafe {
        jl_type_error_rt(fname, context, ty, got);
    }
}

/// No value is bound to this symbol.
pub fn undefined_var_error(var: &Symbol) -> ! {
    let var = var.lock().unwrap();
    unsafe {
        jl_undefined_var_error(var);
    }
}

/// Index ouf of bound.
pub fn bounds_error(v: &Value, idx: &Value) -> ! {
    let v = v.lock().unwrap();
    let idx = idx.lock().unwrap();
    unsafe {
        jl_bounds_error(v, idx);
    }
}

pub fn bounds_error_v(v: &Value, idxs: &[Value]) -> ! {
    let v = v.lock().unwrap();
    let mut indices = SmallVec::<[*mut jl_value_t; 8]>::new();
    for i in idxs {
        indices.push(i.lock().unwrap())
    }
    let nidxs = indices.len();
    let idxs = indices.as_mut_ptr();
    unsafe {
        jl_bounds_error_v(v, idxs, nidxs);
    }
}

/// Index out of bound.
pub fn bounds_error_int(v: &Value, i: usize) -> ! {
    let v = v.lock().unwrap();
    unsafe {
        jl_bounds_error_int(v, i);
    }
}

pub fn bounds_error_tuple_int(v: &[Value], i: usize) -> ! {
    let mut vs = SmallVec::<[*mut jl_value_t; 8]>::new();
    for vi in v {
        vs.push(vi.lock().unwrap());
    }
    let nv = vs.len();
    let v = vs.as_mut_ptr();
    unsafe {
        jl_bounds_error_tuple_int(v, nv, i);
    }
}

// TODO
/*
pub fn bounds_error_unboxed_int(void *v, vt: &Value, i: usize) -> Result<()> {
    let vt = vt.lock()?;
    unsafe {
        jl_bounds_error_unboxed_int();
    }
}
*/

pub fn bounds_error_ints(v: &Value, idxs: &mut [usize]) -> ! {
    let v = v.lock().unwrap();
    let nidxs = idxs.len();
    let idxs = idxs.as_mut_ptr();
    unsafe {
        jl_bounds_error_ints(v, idxs, nidxs);
    }
}

/// Unexpected End of File.
pub fn eof_error() {
    unsafe {
        jl_eof_error();
    }
}
