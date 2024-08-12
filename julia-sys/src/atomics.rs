use std::sync::atomic::{fence, AtomicPtr, Ordering};

pub unsafe fn jl_atomic_load_relaxed<T>(ptr: AtomicPtr<T>) -> *mut T {
    ptr.load(Ordering::Relaxed)
}

pub unsafe fn jl_atomic_store_relaxed<T>(ptr: AtomicPtr<T>, desired: *mut T) {
    ptr.store(desired, Ordering::Relaxed)
}

pub unsafe fn jl_atomic_store_release<T>(ptr: AtomicPtr<T>, desired: *mut T) {
    ptr.store(desired, Ordering::Release)
}

pub unsafe fn jl_signal_fence() {
    fence(Ordering::SeqCst);
}
