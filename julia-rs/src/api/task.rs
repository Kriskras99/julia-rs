//! Module providing a wrapper for the native Julia task object.

use crate::{jlvalues, sys::*};

jlvalues! {
    pub struct Task(jl_task_t);
}

// impl Task {
//     /// Construct a new Task with a Function.
//     pub fn with_function(&self, start: &Function) -> Result<Task> {
//         let raw = unsafe { jl_new_task(start.lock()?, 0) };
//         jl_catch!();
//         Task::new(raw)
//     }
// }
