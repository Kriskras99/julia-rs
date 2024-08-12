use std::sync::atomic::AtomicPtr;

use crate::{
    jl_atomic_load_relaxed, jl_atomic_store_release, jl_ptls_t, jl_signal_fence, JL_GC_STATE_SAFE,
};

#[allow(path_statements)]
pub unsafe fn jl_gc_safepoint_(ptls: jl_ptls_t) {
    jl_signal_fence();
    let _safepoint_load = *(*ptls).safepoint;
    jl_signal_fence();
    _safepoint_load;
}

#[allow(path_statements)]
pub unsafe fn jl_sigint_safepoint(ptls: jl_ptls_t) {
    jl_signal_fence();
    let _safepoint_load = *(*ptls).safepoint.offset(-1);
    jl_signal_fence();
    _safepoint_load;
}

pub unsafe fn jl_gc_state_set(ptls: jl_ptls_t, mut state: i8, old_state: i8) -> i8 {
    jl_atomic_store_release(AtomicPtr::new(&mut (*ptls).gc_state), &mut state);
    // A safe point is required if we transition from GC-safe region to
    // non GC-safe region.
    if old_state != 0 && state == 0 {
        jl_gc_safepoint_(ptls)
    }
    old_state
}

pub unsafe fn jl_gc_state_save_and_set(ptls: jl_ptls_t, state: i8) -> i8 {
    jl_gc_state_set(
        ptls,
        state,
        *jl_atomic_load_relaxed(AtomicPtr::new(&mut (*ptls).gc_state)),
    )
}

pub unsafe fn jl_gc_unsafe_enter(ptls: jl_ptls_t) -> i8 {
    jl_gc_state_save_and_set(ptls, 0)
}

pub unsafe fn jl_gc_unsafe_leave(ptls: jl_ptls_t, state: i8) {
    jl_gc_state_set(ptls, state, 0);
}

pub unsafe fn jl_gc_safe_enter(ptls: jl_ptls_t) -> i8 {
    jl_gc_state_save_and_set(ptls, JL_GC_STATE_SAFE as i8)
}

pub unsafe fn jl_gc_safe_leave(ptls: jl_ptls_t, state: i8) {
    jl_gc_state_set(ptls, state, JL_GC_STATE_SAFE as i8);
}
