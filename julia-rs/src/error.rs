//! This module provides types necessary for error checking and debugging.

use std::char::CharTryFromError;
use std::ffi::{FromBytesWithNulError, IntoStringError, NulError};
use std::fmt;
use std::io;
use std::rc::Rc;
use std::result;
use std::string::FromUtf8Error;
use std::sync::PoisonError;

use crate::api::Exception;

/// Generic julia-rs Result type, used pretty much everywhere a failure might occur
pub type Result<T> = result::Result<T, Error>;

/// A union of all possible errors that might occur in Julia runtime and
/// julia-rs, including Julia exceptions, Rust's io errors and alike, errors
/// arising from trying to use poisonend resources or trying to consume
/// resources in use.
#[derive(Debug)]
pub enum Error {
    /// An exception has occurred.
    UnhandledException(Exception),
    /// Cannot unbox into a certain type.
    InvalidUnbox,
    /// Tried to call a non-function object.
    NotAFunction,
    /// An error occurred while trying to call a function.
    CallError,
    /// An error occurred while evaluating a string or expression.
    EvalError,
    /// Attempt to construct a string or Julia object with a null pointer.
    NullPointer,
    /// Invalid characters used in symbol. See
    /// [docs.julialang.org](https://docs.julialang.org/en/stable/manual/variables/)
    /// for details on symbols and allowed characters.
    InvalidSymbol,
    /// Attempt to initialize Julia in a thread where it's already initialized.
    JuliaInitialized,
    /// Wrapper for ffi::FromBytesWithNulError.
    CStrError(FromBytesWithNulError),
    /// Wrapper for ffi::NulError.
    CStringError(NulError),
    /// Wrapper for sync::PoisonError.
    PoisonError,
    /// Wrapper for errors arising from trying to consume an Rc which is
    /// currently borrowed.
    ResourceInUse,
    /// Wrapper for char::CharTryFromError.
    UTF8Error(CharTryFromError),
    /// Wrapper for string::FromUtf8Error.
    FromUTF8Error(FromUtf8Error),
    /// Wrapper for ffi::IntoStringError.
    IntoStringError(IntoStringError),
    /// Wrapper for io::Error.
    IOError(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::UnhandledException(ref ex) => write!(f, "UnhandledException({})", ex),
            Self::CStrError(ref err) => write!(f, "CStrError({})", err),
            Self::CStringError(ref err) => write!(f, "CStringError({})", err),
            Self::UTF8Error(ref err) => write!(f, "UTF8Error({})", err),
            Self::FromUTF8Error(ref err) => write!(f, "FromUTF8Error({})", err),
            Self::IntoStringError(ref err) => write!(f, "IntoStringError({})", err),
            Self::IOError(ref err) => write!(f, "IOError({})", err),
            Self::InvalidUnbox
            | Self::NotAFunction
            | Self::CallError
            | Self::EvalError
            | Self::NullPointer
            | Self::InvalidSymbol
            | Self::JuliaInitialized
            | Self::PoisonError
            | Self::ResourceInUse => fmt::Debug::fmt(self, f),
        }
    }
}

impl From<FromBytesWithNulError> for Error {
    fn from(err: FromBytesWithNulError) -> Self {
        Self::CStrError(err)
    }
}

impl From<NulError> for Error {
    fn from(err: NulError) -> Self {
        Self::CStringError(err)
    }
}

impl From<CharTryFromError> for Error {
    fn from(err: CharTryFromError) -> Self {
        Self::UTF8Error(err)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Self {
        Self::FromUTF8Error(err)
    }
}

impl<G> From<PoisonError<G>> for Error {
    fn from(_err: PoisonError<G>) -> Self {
        Self::PoisonError
    }
}

impl<T> From<Rc<T>> for Error {
    fn from(_err: Rc<T>) -> Self {
        Self::ResourceInUse
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::IOError(err)
    }
}

impl From<IntoStringError> for Error {
    fn from(err: IntoStringError) -> Self {
        Self::IntoStringError(err)
    }
}
