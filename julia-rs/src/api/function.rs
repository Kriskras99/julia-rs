//! Module providing a wrapper for the native Julia function object.

use smallvec::SmallVec;

use super::{JlValue, Value};
use crate::error::{Error, Result};
use crate::{jlvalues, sys::*};

jlvalues! {
    pub struct Function(jl_function_t);
}

impl Function {
    /// Call with a sequence of Value-s.
    pub fn call<'a, I>(&self, args: I) -> Result<Value>
    where
        I: IntoIterator<Item = &'a Value>,
    {
        let mut argv = SmallVec::<[*mut jl_value_t; 8]>::new();
        for arg in args {
            argv.push(arg.lock()?);
        }

        let ret = unsafe { jl_call(self.lock()?, argv.as_mut_ptr(), argv.len() as u32) };
        jl_catch!();
        Value::new(ret).map_err(|_| Error::CallError)
    }

    /// Call with 0 Value-s.
    pub fn call0(&self) -> Result<Value> {
        let ret = unsafe { jl_call0(self.lock()?) };
        jl_catch!();
        Value::new(ret).map_err(|_| Error::CallError)
    }

    /// Call with 1 Value.
    pub fn call1(&self, arg1: &Value) -> Result<Value> {
        let ret = unsafe { jl_call1(self.lock()?, arg1.lock()?) };
        jl_catch!();
        Value::new(ret).map_err(|_| Error::CallError)
    }

    /// Call with 2 Value-s.
    pub fn call2(&self, arg1: &Value, arg2: &Value) -> Result<Value> {
        let ret = unsafe { jl_call2(self.lock()?, arg1.lock()?, arg2.lock()?) };
        jl_catch!();
        Value::new(ret).map_err(|_| Error::CallError)
    }

    /// Call with 3 Value-s.
    pub fn call3(&self, arg1: &Value, arg2: &Value, arg3: &Value) -> Result<Value> {
        let ret = unsafe { jl_call3(self.lock()?, arg1.lock()?, arg2.lock()?, arg3.lock()?) };
        jl_catch!();
        Value::new(ret).map_err(|_| Error::CallError)
    }
}
