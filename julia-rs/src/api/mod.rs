//! Main entry point to the Julia api.

use std::ffi::CString;
use std::io::Read;

use crate::error::{Error, Result};
use crate::string::IntoCString;
use crate::sys::*;
use crate::version::Version;

/// This macro checks for exceptions that might have occurred in the sys::*
/// functions. Should be used after calling any jl_* function that might throw
/// an exception.
#[macro_export]
macro_rules! jl_catch {
    () => {
        $crate::jl_catch!(|ex| { ex });
    };
    (|$ex:ident| $body:expr) => {
        $crate::jl_catch!(|$ex -> $crate::error::Error::UnhandledException| $crate::error::Error::UnhandledException($body));
    };
    (|$ex:ident -> $t:ty| $body:expr) => {
        #[allow(unused_variables)] // this shouldn't be necessary
        {
            if let Some($ex) = $crate::api::Exception::catch() {
                return Err($body);
            }
        }
    }
}

pub mod array;
pub mod datatype;
pub mod exception;
pub mod function;
pub mod module;
pub mod primitive;
pub mod sym;
pub mod task;
pub mod value;

pub use self::array::{Array, Svec};
pub use self::datatype::Datatype;
pub use self::exception::Exception;
pub use self::function::Function;
pub use self::module::Module;
pub use self::primitive::*;
pub use self::sym::{IntoSymbol, Symbol};
pub use self::task::Task;
pub use self::value::{JlValue, Value};

/// Blank struct for controlling the Julia garbage collector.
pub struct Gc;

impl Gc {
    /// Enable or disable the garbage collector.
    pub fn enable(&mut self, p: bool) -> Result<()> {
        unsafe {
            jl_gc_enable(p as i32);
        }
        jl_catch!();
        Ok(())
    }

    /// Check to see if gc is enabled.
    pub fn is_enabled(&self) -> bool {
        unsafe { jl_gc_is_enabled() != 0 }
    }

    /// Collect immediately. Set full to true if a full garbage collection
    /// should be issued
    pub fn collect(&mut self, full: bool) -> Result<()> {
        unsafe {
            jl_gc_collect(full as u32);
        }
        jl_catch!();
        Ok(())
    }
}

/// Struct for controlling the Julia runtime.
pub struct Julia {
    main: Module,
    core: Module,
    base: Module,
    top: Module,
    at_exit: Option<i32>,
    gc: Gc,
}

impl Julia {
    /// Assume that Julia was already initialized somewhere else and return a
    /// handle.
    ///
    /// # Safety
    /// Julia needs to be initialized otherwise segfaults are likely to happen.
    ///
    /// ## Panics
    ///
    /// Panics if the Julia runtime was not previously initialized.
    pub unsafe fn new_unchecked() -> Self {
        if !Self::is_initialized() {
            panic!("Julia is not initialized");
        }

        let main = Module::new_unchecked(jl_main_module);
        let core = Module::new_unchecked(jl_core_module);
        let base = Module::new_unchecked(jl_base_module);
        let top = Module::new_unchecked(jl_top_module);

        Self {
            main,
            core,
            base,
            top,
            at_exit: None,
            gc: Gc,
        }
    }

    /// Initialize the Julia runtime.
    ///
    /// ## Errors
    ///
    /// Returns Error::JuliaInitialized if Julia is already initialized.
    pub fn new() -> Result<Self> {
        if Self::is_initialized() {
            return Err(Error::JuliaInitialized);
        }

        unsafe {
            jl_init();
        }
        jl_catch!();

        let mut jl = unsafe { Self::new_unchecked() };
        jl.at_exit = Some(0);
        Ok(jl)
    }

    /// Initialize the Julia runtime with a specific sysimage.
    ///
    /// ## Errors
    ///
    /// Returns Error::JuliaInitialized if Julia is already initialized.
    pub fn new_with_image(image_path: &str) -> Result<Self> {
        if Self::is_initialized() {
            return Err(Error::JuliaInitialized);
        }

        let image_path = CString::new(image_path).unwrap();

        unsafe {
            jl_init_with_image(std::ptr::null(), image_path.as_ptr());
        }
        jl_catch!();

        let mut jl = unsafe { Self::new_unchecked() };
        jl.at_exit = Some(0);
        Ok(jl)
    }

    /// Returns the version of currently running Julia runtime.
    pub fn version(&self) -> Version {
        unsafe {
            let major = jl_ver_major() as u32;
            let minor = jl_ver_minor() as u32;
            let patch = jl_ver_patch() as u32;
            let release = jl_ver_is_release() != 0;

            Version {
                name: "julia",
                major,
                minor,
                patch,
                release,
            }
        }
    }

    /// Returns a reference to the garbage collector.
    pub const fn gc(&self) -> &Gc {
        &self.gc
    }

    /// Returns a mutable reference to the garbage collector.
    pub fn gc_mut(&mut self) -> &mut Gc {
        &mut self.gc
    }

    /// Checks if Julia was already initialized in the current thread.
    pub fn is_initialized() -> bool {
        unsafe { jl_is_initialized() != 0 }
    }

    /// Sets status to at_exit and consumes Julia, causing the value to be
    /// dropped.
    pub fn exit(mut self, at_exit: i32) {
        self.at_exit(Some(at_exit))
    }

    /// Sets status.
    pub fn at_exit(&mut self, at_exit: Option<i32>) {
        self.at_exit = at_exit;
    }

    /// Returns a handle to the main module.
    pub const fn main(&self) -> &Module {
        &self.main
    }

    /// Returns a handle to the core module.
    pub const fn core(&self) -> &Module {
        &self.core
    }

    /// Returns a handle to the base module.
    pub const fn base(&self) -> &Module {
        &self.base
    }

    /// Returns a handle to the top module.
    pub const fn top(&self) -> &Module {
        &self.top
    }

    /// Loads a Julia script from any Read without evaluating it.
    pub fn load<R: Read, S: IntoCString>(&mut self, r: &mut R, name: Option<S>) -> Result<Value> {
        let mut content = String::new();
        let len = r.read_to_string(&mut content)?;
        let content = content.into_cstring();
        let content = content.as_ptr();

        let name = name
            .map(|s| s.into_cstring())
            .unwrap_or_else(|| "string".into_cstring());
        let name = name.as_ptr();

        //let raw = unsafe { jl_load_file_string(content, len, ptr::null::<i8>() as *mut _) };
        let raw = unsafe { jl_load_file_string(content, len, name as *mut _, jl_main_module) };
        jl_catch!();
        Value::new(raw)
    }

    /// Parses and evaluates string.
    pub fn eval_string<S: IntoCString>(&mut self, string: S) -> Result<Value> {
        let string = string.into_cstring();
        let string = string.as_ptr();

        let ret = unsafe { jl_eval_string(string) };
        jl_catch!();
        Value::new(ret).map_err(|_| Error::EvalError)
    }
}

impl Drop for Julia {
    fn drop(&mut self) {
        if let Some(s) = self.at_exit {
            unsafe { jl_atexit_hook(s) }
        }
    }
}
