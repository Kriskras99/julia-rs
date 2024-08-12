#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::all)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::ffi::{c_char, c_void};
use std::{mem::offset_of, sync::atomic::AtomicPtr};

mod atomics;
mod threads;
pub use atomics::*;
pub use threads::*;

pub unsafe fn jl_astaggedvalue<T>(v: *const T) -> *const jl_taggedvalue_t {
    v.byte_sub(size_of::<jl_taggedvalue_t>()) as *const jl_taggedvalue_t
}

pub unsafe fn jl_valueof<T>(v: *const T) -> *const jl_value_t {
    v.byte_add(size_of::<jl_taggedvalue_t>()) as *const jl_value_t
}

pub unsafe fn jl_typeof<T>(v: *const T) -> *const jl_value_t {
    jl_to_typeof(jl_typetagof(v))
}

pub unsafe fn jl_typetagof<T>(v: *const T) -> usize {
    (*jl_astaggedvalue(v)).__bindgen_anon_1.header & (!15_usize)
}

pub unsafe fn jl_typeis<T, U>(v: *const T, t: *const U) -> bool {
    jl_typeof(v) == (t as *const jl_value_t)
}

pub unsafe fn jl_typetagis<T>(v: *const T, t: usize) -> bool {
    jl_typetagof(v) == t
}

/// compute # of extra words needed to store dimensions
pub unsafe fn jl_array_ndimwords(ndims: usize) -> usize {
    if ndims < 3 {
        0
    } else {
        ndims - 2
    }
}

pub unsafe fn jl_to_typeof(t: usize) -> *const jl_value_t {
    if t < ((jl_small_typeof_tags_jl_max_tags << 4) as usize) {
        return jl_small_typeof[t / (jl_small_typeof.len() * size_of::<*const jl_datatype_t>())]
            as *const jl_value_t;
    }
    t as *const jl_value_t
}

pub unsafe fn jl_current_task() -> *mut jl_task_t {
    let ptr = jl_get_pgcstack();
    let ptr = ptr.byte_sub(offset_of!(jl_task_t, gcstack));
    ptr as *mut jl_task_t
}

pub unsafe fn jl_pgcstack() -> *const jl_gcframe_t {
    (*jl_current_task()).gcstack
}

pub unsafe fn jl_set_pgcstack(gcstack: *mut jl_gcframe_t) {
    (*jl_current_task()).gcstack = gcstack
}

pub unsafe fn read_the_macro_documentation(_u: &'static str) {}

/// Push one or more values onto Julia's GC stack
///
/// This keeps a variable alive across multiple calls into Julia.
/// You need to call [`JL_GC_POP`] *before* the current scope ends,
/// this is because this macro creates references to the current stack.
/// It should only be called once per scope!
///
/// To use this macro you need to wrap it in an unsafe block.
// TODO: Maybe we can replace the local array with a Vec? Does Julia care if it actually is on a stack?
#[macro_export]
macro_rules! JL_GC_PUSH {
    ($( $arg1:expr),+) => {
        $crate::read_the_macro_documentation("This macro is very unsafe and cannot check its prerequisites");

        let mut __count = 0usize;
        $( || $arg1;
            __count += 1;
        ),+

        let __encode_push = if __count <= 8 {
            (__count << 2) | 1
        } else {
            __count << 2
        };

        unsafe {
            let __gc_stckf = [
                __encode_push as *const ::std::ffi::c_void,
                (*$crate::jl_get_current_task()).gcstack as *const ::std::ffi::c_void,
                $( ($arg1) as *const ::std::ffi::c_void, ),+
            ].as_mut_ptr() as *mut $crate::jl_gcframe_t;
            $crate::jl_set_pgcstack(__gc_stckf);
        }
    };
}

/// Pop one or more values from Julia's GC stack
///
/// This ends the life of a variable.
/// You need to call [`JL_GC_PUSH`] *before* this macro and in the same scope,
/// otherwise your removing Julia's GC stackframes.
/// It should only be called once per scope!
///
/// To use this macro you need to wrap it in an unsafe block.
// TODO: Maybe we can replace the local array with a Vec? Does Julia care if it actually is on a stack?
#[macro_export]
macro_rules! JL_GC_POP {
    () => {
        $crate::read_the_macro_documentation(
            "This macro is very unsafe and cannot check its prerequisites",
        );
        let __prev_gc_stckf = (*jl_pgcstack()).prev;
        jl_set_pgcstack(__prev_gc_stckf);
    };
}

pub unsafe fn jl_gc_wb<T, U>(parent: *const T, ptr: *const U) {
    // parent and ptr isa jl_value_t*
    if (*jl_astaggedvalue(parent)).__bindgen_anon_1.bits.gc() == 3
        && ((*jl_astaggedvalue(ptr)).__bindgen_anon_1.bits.gc() & 1) == 0
    {
        jl_gc_queue_root(parent as *const jl_value_t);
    }
}

pub unsafe fn jl_gc_wb_back<T>(ptr: *const T) {
    // if ptr is old
    if (*jl_astaggedvalue(ptr)).__bindgen_anon_1.bits.gc() == 3 {
        jl_gc_queue_root(ptr as *const jl_value_t);
    }
}

pub unsafe fn jl_gc_multi_wb<T, U>(parent: *const T, ptr: *const U) {
    // ptr is an immutable object
    if (*jl_astaggedvalue(parent)).__bindgen_anon_1.bits.gc() != 3 {
        return; // parent is young or in remset
    } else if (*jl_astaggedvalue(parent)).__bindgen_anon_1.bits.gc() == 3 {
        return; // ptr is old and not in remset (thus it does not point to young)
    }
    let dt = jl_typeof(ptr) as *const jl_datatype_t;
    let ly = (*dt).layout;
    if (*ly).npointers != 0 {
        jl_gc_queue_multiroot(parent as *const jl_value_t, ptr as *const jl_value_t);
    }
}

pub unsafe fn jl_svec_len(t: *const jl_svec_t) -> usize {
    (*t).length
}

pub unsafe fn jl_svec_set_len_unsafe(t: *mut jl_svec_t, n: usize) {
    (*t).length = n;
}

pub unsafe fn jl_svec_data(t: *mut jl_svec_t) -> *mut *mut jl_value_t {
    t.add(1) as *mut *mut jl_value_t
}

pub unsafe fn jl_svecref<T>(t: *mut T, i: usize) -> *mut jl_value_t {
    debug_assert!(jl_typetagis(
        t,
        (jl_small_typeof_tags_jl_simplevector_tag << 4) as usize
    ));
    debug_assert!(i < jl_svec_len(t as *mut jl_svec_t));
    // missing atomics
    jl_atomic_load_relaxed(AtomicPtr::new(
        *jl_svec_data(t as *mut jl_svec_t).byte_add(i),
    ))
}

pub unsafe fn jl_svecset<T, U>(t: *mut T, i: usize, x: *mut U) -> *mut jl_value_t {
    debug_assert!(jl_typetagis(
        t,
        (jl_small_typeof_tags_jl_simplevector_tag << 4) as usize
    ));
    assert!(i < jl_svec_len(t as *mut jl_svec_t));
    jl_atomic_store_relaxed(
        AtomicPtr::new(jl_svec_data(t as *mut jl_svec_t).byte_add(i) as *mut jl_value_t),
        x as *mut jl_value_t,
    );
    jl_gc_wb(t, x);
    x as *mut jl_value_t
}

pub unsafe fn jl_array_len(a: *const jl_array_t) -> usize {
    (*a).length
}

pub unsafe fn jl_array_data(a: *mut jl_array_t) -> *mut c_void {
    (*a).data
}

pub unsafe fn jl_array_dim(a: *const jl_array_t, i: usize) -> usize {
    *((&(*a).nrows) as *const usize).add(i)
}

pub unsafe fn jl_array_dim0(a: *const jl_array_t) -> usize {
    (*a).nrows
}

pub unsafe fn jl_array_nrows(a: *const jl_array_t) -> usize {
    (*a).nrows
}

pub unsafe fn jl_array_ndims(a: *const jl_array_t) -> usize {
    (*a).flags.ndims() as usize
}

/// in bytes
pub unsafe fn jl_array_data_owner_offset(ndims: usize) -> usize {
    offset_of!(jl_array_t, __bindgen_anon_1) + size_of::<usize>() * (1 + jl_array_ndimwords(ndims))
}

pub unsafe fn jl_array_data_owner(a: *mut jl_array_t) -> *mut jl_value_t {
    *(a.byte_add(jl_array_data_owner_offset(jl_array_ndims(a))) as *mut *mut jl_value_t)
}

pub unsafe fn jl_array_ptr_data(a: *mut jl_array_t) -> *mut *mut jl_value_t {
    (*a).data as *mut *mut jl_value_t
}

pub unsafe fn jl_array_ptr_ref(a: *mut jl_array_t, i: usize) -> *mut jl_value_t {
    debug_assert!((*a).flags.ptrarray() != 0);
    debug_assert!(i < jl_array_len(a));
    jl_atomic_load_relaxed(AtomicPtr::new(
        jl_array_data(a).byte_add(i) as *mut jl_value_t
    ))
}

pub unsafe fn jl_array_ptr_set<U>(mut a: *mut jl_array_t, i: usize, x: *mut U) -> *mut jl_value_t {
    debug_assert!((*a).flags.ptrarray() != 0);
    debug_assert!(i < jl_array_len(a));
    jl_atomic_store_relaxed(
        AtomicPtr::new((jl_array_data(a) as *mut *mut jl_value_t).byte_add(i) as *mut jl_value_t),
        x as *mut jl_value_t,
    );
    if !x.is_null() {
        if (*a).flags.how() == 3 {
            a = jl_array_data_owner(a) as *mut jl_array_t;
        }
        jl_gc_wb(a, x);
    }
    x as *mut jl_value_t
}

pub unsafe fn jl_array_uint8_ref(a: *mut jl_array_t, i: usize) -> u8 {
    debug_assert!(i < jl_array_len(a));
    debug_assert!(jl_typetagis(a, jl_array_uint8_type as usize));
    *(jl_array_data(a) as *mut u8).add(i)
}

pub unsafe fn jl_array_uint8_set(a: *mut jl_array_t, i: usize, x: u8) {
    debug_assert!(i < jl_array_len(a));
    debug_assert!(jl_typetagis(a, jl_array_uint8_type as usize));
    *(jl_array_data(a) as *mut u8).add(i) = x;
}

pub unsafe fn jl_exprarg<T>(e: *mut T, n: usize) -> *mut jl_value_t {
    jl_array_ptr_ref((*(e as *mut jl_expr_t)).args, n)
}

pub unsafe fn jl_exprargset<T, U>(e: *mut T, n: usize, v: *mut U) -> *mut jl_value_t {
    jl_array_ptr_set((*(e as *mut jl_expr_t)).args, n, v)
}

pub unsafe fn jl_expr_nargs<T>(e: *mut T) -> usize {
    jl_array_len((*(e as *mut jl_expr_t)).args)
}

pub unsafe fn jl_fieldref<T>(s: *mut T, i: usize) -> *mut jl_value_t {
    jl_get_nth_field(s as *mut jl_value_t, i)
}

pub unsafe fn jl_fieldref_noalloc<T>(s: *mut T, i: usize) -> *mut jl_value_t {
    jl_get_nth_field_noalloc(s as *mut jl_value_t, i)
}

pub unsafe fn jl_nfields<T>(v: *mut T) -> usize {
    jl_datatype_nfields(jl_typeof(v as *const jl_value_t) as *const jl_datatype_t) as usize
}

// Not using jl_fieldref to avoid allocations
pub unsafe fn jl_linenode_line<T>(x: *mut T) -> isize {
    *(x as *mut isize)
}

pub unsafe fn jl_linenode_file<T>(x: *mut T) -> *mut jl_value_t {
    *(x as *mut *mut jl_value_t).add(1)
}

pub unsafe fn jl_slot_number<T>(x: *mut T) -> isize {
    *(x as *mut isize)
}

pub unsafe fn jl_typedslot_get_type<T>(x: *mut T) -> *mut jl_value_t {
    *(x as *mut *mut jl_value_t).add(1)
}

pub unsafe fn jl_gotonode_label<T>(x: *mut T) -> isize {
    *(x as *mut isize)
}

pub unsafe fn jl_gotoifnot_cond<T>(x: *mut T) -> *mut jl_value_t {
    *(x as *mut *mut jl_value_t)
}

pub unsafe fn jl_gotoifnot_label<T>(x: *mut T) -> isize {
    *(x as *mut isize).add(1)
}

pub unsafe fn jl_globalref_mod<T>(s: *mut T) -> *mut jl_module_t {
    *(s as *mut *mut jl_module_t)
}

pub unsafe fn jl_globalref_name<T>(s: *mut T) -> *mut jl_sym_t {
    *(s as *mut *mut jl_sym_t).add(1)
}

pub unsafe fn jl_quotenode_value<T>(x: *mut T) -> *mut jl_value_t {
    *(x as *mut *mut jl_value_t)
}

pub unsafe fn jl_returnnode_value<T>(x: *mut T) -> *mut jl_value_t {
    *(x as *mut *mut jl_value_t)
}

pub unsafe fn jl_nparams<T>(t: *mut T) -> usize {
    jl_svec_len((*(t as *mut jl_datatype_t)).parameters)
}

pub unsafe fn jl_tparam0<T>(t: *mut T) -> *mut jl_value_t {
    jl_tparam(t, 0)
}

pub unsafe fn jl_tparam1<T>(t: *mut T) -> *mut jl_value_t {
    jl_tparam(t, 1)
}

pub unsafe fn jl_tparam<T>(t: *mut T, i: usize) -> *mut jl_value_t {
    jl_svecref((*(t as *mut jl_datatype_t)).parameters, i)
}

// get a pointer to the data in a datatype
pub unsafe fn jl_data_ptr<T>(v: *mut T) -> *mut *mut jl_value_t {
    v as *mut *mut jl_value_t
}

pub unsafe fn jl_string_data<T>(s: *mut T) -> *mut c_char {
    (s as *mut c_char).byte_add(size_of::<*const c_void>())
}

pub unsafe fn jl_string_len<T>(s: *mut T) -> usize {
    *(s as *mut usize)
}

pub unsafe fn jl_gf_mtable<T>(f: *mut T) -> *mut jl_methtable_t {
    (*((*(jl_typeof(f) as *mut jl_datatype_t)).name)).mt
}

pub unsafe fn jl_gf_name<T>(f: *mut T) -> *mut jl_sym_t {
    (*jl_gf_mtable(f)).name
}

// struct type info
pub unsafe fn jl_get_fieldtypes(st: *mut jl_datatype_t) -> *mut jl_svec_t {
    if (*st).types.is_null() {
        jl_compute_fieldtypes(st, std::ptr::null_mut())
    } else {
        (*st).types
    }
}

pub unsafe fn jl_field_names(st: *mut jl_datatype_t) -> *mut jl_svec_t {
    (*(*st).name).names
}

pub unsafe fn jl_field_type(st: *mut jl_datatype_t, i: usize) -> *mut jl_value_t {
    jl_svecref(jl_get_fieldtypes(st), i)
}

pub unsafe fn jl_field_type_concrete(st: *mut jl_datatype_t, i: usize) -> *mut jl_value_t {
    debug_assert!(!(*st).types.is_null());
    jl_svecref((*st).types, i)
}

pub unsafe fn jl_datatype_size(t: *const jl_datatype_t) -> usize {
    (*(*t).layout).size as usize
}

pub unsafe fn jl_datatype_align(t: *const jl_datatype_t) -> usize {
    (*(*t).layout).alignment as usize
}

pub unsafe fn jl_datatype_nbits(t: *const jl_datatype_t) -> usize {
    jl_datatype_size(t) * 8
}

pub unsafe fn jl_datatype_nfields(t: *const jl_datatype_t) -> usize {
    (*(*t).layout).nfields as usize
}

// from julia/dtypes.h:
// #define LLT_ALIGN(x, sz) (((x) + (sz)-1) & ~((sz)-1))
fn LLT_ALIGN(x: usize, sz: usize) -> usize {
    (x + sz - 1) & !(sz - 1)
}

pub unsafe fn jl_symbol_name_(s: *mut jl_sym_t) -> *mut c_char {
    (s as *mut c_char).byte_add(LLT_ALIGN(size_of::<jl_sym_t>(), size_of::<*const c_void>()))
}

pub unsafe fn jl_fielddesc_size(fielddesc_type: i8) -> u32 {
    debug_assert!(fielddesc_type >= 0 && fielddesc_type <= 2);
    2u32 << fielddesc_type
}

pub unsafe fn jl_dt_layout_fields<T>(d: *const T) -> *const c_char {
    (d as *const c_char).byte_add(size_of::<jl_datatype_layout_t>())
}

pub unsafe fn jl_dt_layout_ptrs(l: *const jl_datatype_layout_t) -> *const c_char {
    jl_dt_layout_fields(l)
        .byte_add((jl_fielddesc_size((*l).fielddesc_type() as i8) * (*l).nfields) as usize)
}

pub unsafe fn jl_field_offset(st: *mut jl_datatype_t, i: usize) -> usize {
    let ly = (*st).layout;
    debug_assert!(i < (*ly).nfields as usize);

    match (*ly).fielddesc_type() {
        0 => (*(jl_dt_layout_fields(ly) as *mut jl_fielddesc8_t).add(i)).offset as usize,
        1 => (*(jl_dt_layout_fields(ly) as *mut jl_fielddesc16_t).add(i)).offset as usize,
        2 => (*(jl_dt_layout_fields(ly) as *mut jl_fielddesc32_t).add(i)).offset as usize,
        _ => panic!(),
    }
}

pub unsafe fn jl_field_size(st: *mut jl_datatype_t, i: usize) -> usize {
    let ly = (*st).layout;
    debug_assert!(i < (*ly).nfields as usize);

    match (*ly).fielddesc_type() {
        0 => (*(jl_dt_layout_fields(ly) as *mut jl_fielddesc8_t).add(i)).size() as usize,
        1 => (*(jl_dt_layout_fields(ly) as *mut jl_fielddesc16_t).add(i)).size() as usize,
        2 => (*(jl_dt_layout_fields(ly) as *mut jl_fielddesc32_t).add(i)).size() as usize,
        _ => panic!(),
    }
}

pub unsafe fn jl_field_isptr(st: *mut jl_datatype_t, i: isize) -> bool {
    let ly = (*st).layout;
    debug_assert!(i >= 0 && i < (*ly).nfields as isize);
    (*(jl_dt_layout_fields(ly) as *const jl_fielddesc8_t)
        .byte_add((jl_fielddesc_size((*ly).fielddesc_type() as i8) * i as u32) as usize))
    .isptr()
        != 0
}

pub unsafe fn jl_ptr_offset(st: *mut jl_datatype_t, i: isize) -> u32 {
    let ly = (*st).layout;
    debug_assert!(i >= 0 && i < (*ly).npointers as isize);
    let ptrs = jl_dt_layout_ptrs(ly);

    match (*ly).fielddesc_type() {
        0 => (*(ptrs as *const u8).byte_add(i as usize)) as u32,
        1 => (*(ptrs as *const u16).byte_add(i as usize)) as u32,
        2 => *(ptrs as *const u32).byte_add(i as usize),
        _ => panic!(),
    }
}

pub unsafe fn jl_field_isatomic(st: *mut jl_datatype_t, i: usize) -> bool {
    let atomicfields = (*(*st).name).atomicfields;
    if !atomicfields.is_null() {
        if (atomicfields.add(i / 32) as usize) & (1 << (i % 32)) != 0 {
            return true;
        }
    }
    false
}

pub unsafe fn jl_field_isconst(st: *mut jl_datatype_t, i: usize) -> bool {
    let tn = (*st).name;
    if (*tn).mutabl() == 0 {
        return true;
    }
    let constfields = (*tn).constfields;
    if !constfields.is_null() {
        if (constfields.add(i / 32) as usize) & (1 << (i % 32)) != 0 {
            return true;
        }
    }
    false
}

pub unsafe fn jl_is_layout_opaque(l: *const jl_datatype_layout_t) -> bool {
    (*l).nfields == 0 && (*l).npointers > 0
}

pub unsafe fn jl_is_nothing<T>(v: *const T) -> bool {
    (v as *const jl_value_t) == jl_nothing
}
pub unsafe fn jl_is_tuple<T>(v: *const T) -> bool {
    (*(jl_typeof(v) as *const jl_datatype_t)).name == jl_tuple_typename
}
pub unsafe fn jl_is_namedtuple<T>(v: *const T) -> bool {
    (*(jl_typeof(v) as *const jl_datatype_t)).name == jl_namedtuple_typename
}
pub unsafe fn jl_is_svec<T>(v: *const T) -> bool {
    jl_typetagis(v, (jl_small_typeof_tags_jl_simplevector_tag << 4) as usize)
}
pub unsafe fn jl_is_simplevector<T>(v: *const T) -> bool {
    jl_is_svec(v)
}
pub unsafe fn jl_is_datatype<T>(v: *const T) -> bool {
    jl_typetagis(v, (jl_small_typeof_tags_jl_datatype_tag << 4) as usize)
}
pub unsafe fn jl_is_mutable<T>(t: *const T) -> bool {
    (*(*(t as *const jl_datatype_t)).name).mutabl() != 0
}
pub unsafe fn jl_is_mutable_datatype<T>(t: *const T) -> bool {
    jl_is_datatype(t) && jl_is_mutable(t)
}
pub unsafe fn jl_is_immutable<T>(t: *const T) -> bool {
    !jl_is_mutable(t)
}
pub unsafe fn jl_is_immutable_datatype<T>(t: *const T) -> bool {
    jl_is_datatype(t) && !jl_is_mutable(t)
}
pub unsafe fn jl_is_uniontype<T>(v: *const T) -> bool {
    jl_typetagis(v, (jl_small_typeof_tags_jl_uniontype_tag << 4) as usize)
}
pub unsafe fn jl_is_typevar<T>(v: *const T) -> bool {
    jl_typetagis(v, (jl_small_typeof_tags_jl_tvar_tag << 4) as usize)
}
pub unsafe fn jl_is_unionall<T>(v: *const T) -> bool {
    jl_typetagis(v, (jl_small_typeof_tags_jl_unionall_tag << 4) as usize)
}
pub unsafe fn jl_is_vararg<T>(v: *const T) -> bool {
    jl_typetagis(v, (jl_small_typeof_tags_jl_vararg_tag << 4) as usize)
}
pub unsafe fn jl_is_typename<T>(v: *const T) -> bool {
    jl_typetagis(v, jl_typename_type as usize)
}
pub unsafe fn jl_is_int8<T>(v: *const T) -> bool {
    jl_typetagis(v, (jl_small_typeof_tags_jl_int8_tag << 4) as usize)
}
pub unsafe fn jl_is_int16<T>(v: *const T) -> bool {
    jl_typetagis(v, (jl_small_typeof_tags_jl_int16_tag << 4) as usize)
}
pub unsafe fn jl_is_int32<T>(v: *const T) -> bool {
    jl_typetagis(v, (jl_small_typeof_tags_jl_int32_tag << 4) as usize)
}
pub unsafe fn jl_is_int64<T>(v: *const T) -> bool {
    jl_typetagis(v, (jl_small_typeof_tags_jl_int64_tag << 4) as usize)
}
pub unsafe fn jl_is_uint8<T>(v: *const T) -> bool {
    jl_typetagis(v, (jl_small_typeof_tags_jl_uint8_tag << 4) as usize)
}
pub unsafe fn jl_is_uint16<T>(v: *const T) -> bool {
    jl_typetagis(v, (jl_small_typeof_tags_jl_uint16_tag << 4) as usize)
}
pub unsafe fn jl_is_uint32<T>(v: *const T) -> bool {
    jl_typetagis(v, (jl_small_typeof_tags_jl_uint32_tag << 4) as usize)
}
pub unsafe fn jl_is_uint64<T>(v: *const T) -> bool {
    jl_typetagis(v, (jl_small_typeof_tags_jl_uint64_tag << 4) as usize)
}
pub unsafe fn jl_is_bool<T>(v: *const T) -> bool {
    jl_typetagis(v, (jl_small_typeof_tags_jl_bool_tag << 4) as usize)
}
pub unsafe fn jl_is_symbol<T>(v: *const T) -> bool {
    jl_typetagis(v, (jl_small_typeof_tags_jl_symbol_tag << 4) as usize)
}
pub unsafe fn jl_is_ssavalue<T>(v: *const T) -> bool {
    jl_typetagis(v, jl_ssavalue_type as usize)
}
pub unsafe fn jl_is_slotnumber<T>(v: *const T) -> bool {
    jl_typetagis(v, jl_slotnumber_type as usize)
}
pub unsafe fn jl_is_expr<T>(v: *const T) -> bool {
    jl_typetagis(v, jl_expr_type as usize)
}
pub unsafe fn jl_is_binding<T>(v: *const T) -> bool {
    jl_typetagis(v, jl_binding_type as usize)
}
pub unsafe fn jl_is_globalref<T>(v: *const T) -> bool {
    jl_typetagis(v, jl_globalref_type as usize)
}
pub unsafe fn jl_is_gotonode<T>(v: *const T) -> bool {
    jl_typetagis(v, jl_gotonode_type as usize)
}
pub unsafe fn jl_is_gotoifnot<T>(v: *const T) -> bool {
    jl_typetagis(v, jl_gotoifnot_type as usize)
}
pub unsafe fn jl_is_returnnode<T>(v: *const T) -> bool {
    jl_typetagis(v, jl_returnnode_type as usize)
}
pub unsafe fn jl_is_argument<T>(v: *const T) -> bool {
    jl_typetagis(v, jl_argument_type as usize)
}
pub unsafe fn jl_is_pinode<T>(v: *const T) -> bool {
    jl_typetagis(v, jl_pinode_type as usize)
}
pub unsafe fn jl_is_phinode<T>(v: *const T) -> bool {
    jl_typetagis(v, jl_phinode_type as usize)
}
pub unsafe fn jl_is_phicnode<T>(v: *const T) -> bool {
    jl_typetagis(v, jl_phicnode_type as usize)
}
pub unsafe fn jl_is_upsilonnode<T>(v: *const T) -> bool {
    jl_typetagis(v, jl_upsilonnode_type as usize)
}
pub unsafe fn jl_is_quotenode<T>(v: *const T) -> bool {
    jl_typetagis(v, jl_quotenode_type as usize)
}
pub unsafe fn jl_is_newvarnode<T>(v: *const T) -> bool {
    jl_typetagis(v, jl_newvarnode_type as usize)
}
pub unsafe fn jl_is_linenode<T>(v: *const T) -> bool {
    jl_typetagis(v, jl_linenumbernode_type as usize)
}
pub unsafe fn jl_is_method_instance<T>(v: *const T) -> bool {
    jl_typetagis(v, jl_method_instance_type as usize)
}
pub unsafe fn jl_is_code_instance<T>(v: *const T) -> bool {
    jl_typetagis(v, jl_code_instance_type as usize)
}
pub unsafe fn jl_is_code_info<T>(v: *const T) -> bool {
    jl_typetagis(v, jl_code_info_type as usize)
}
pub unsafe fn jl_is_method<T>(v: *const T) -> bool {
    jl_typetagis(v, jl_method_type as usize)
}
pub unsafe fn jl_is_module<T>(v: *const T) -> bool {
    jl_typetagis(v, (jl_small_typeof_tags_jl_module_tag << 4) as usize)
}
pub unsafe fn jl_is_mtable<T>(v: *const T) -> bool {
    jl_typetagis(v, jl_methtable_type as usize)
}
pub unsafe fn jl_is_task<T>(v: *const T) -> bool {
    jl_typetagis(v, (jl_small_typeof_tags_jl_task_tag << 4) as usize)
}
pub unsafe fn jl_is_string<T>(v: *const T) -> bool {
    jl_typetagis(v, (jl_small_typeof_tags_jl_string_tag << 4) as usize)
}
pub unsafe fn jl_is_cpointer<T>(v: *const T) -> bool {
    jl_is_cpointer_type(jl_typeof(v))
}
pub unsafe fn jl_is_pointer<T>(v: *const T) -> bool {
    jl_is_cpointer_type(jl_typeof(v))
}
pub unsafe fn jl_is_uint8pointer<T>(v: *const T) -> bool {
    jl_typetagis(v, jl_uint8pointer_type as usize)
}
pub unsafe fn jl_is_llvmpointer<T>(v: *const T) -> bool {
    (*(jl_typeof(v) as *const jl_datatype_t)).name == jl_llvmpointer_typename
}
pub unsafe fn jl_is_intrinsic<T>(v: *const T) -> bool {
    jl_typeis(v, jl_intrinsic_type)
}

pub unsafe fn jl_array_isbitsunion<T>(a: *const T) -> bool {
    (*(a as *const jl_array_t)).flags.ptrarray() == 0
        && jl_is_uniontype(jl_tparam0(jl_typeof(a) as *mut jl_value_t))
}

pub unsafe fn jl_is_kind(v: *const jl_value_t) -> bool {
    v == jl_uniontype_type as *const jl_value_t
        || v == jl_datatype_type as *const jl_value_t
        || v == jl_unionall_type as *const jl_value_t
        || v == jl_typeofbottom_type as *const jl_value_t
}

pub unsafe fn jl_is_kindtag(t: usize) -> bool {
    let t = t >> 4;
    t == jl_uniontype_type as usize
        || t == jl_datatype_type as usize
        || t == jl_unionall_type as usize
        || t == jl_typeofbottom_type as usize
}

pub unsafe fn jl_is_type<T>(v: *const T) -> bool {
    jl_is_kindtag(jl_typetagof(v))
}

pub unsafe fn jl_is_primitivetype<T>(v: *const T) -> bool {
    jl_is_datatype(v) && (*(v as *const jl_datatype_t)).isprimitivetype() != 0
}

pub unsafe fn jl_is_structtype<T>(v: *const T) -> bool {
    jl_is_datatype(v)
        && (*(*(v as *const jl_datatype_t)).name).abstract_() == 0
        && (*(v as *const jl_datatype_t)).isprimitivetype() == 0
}

/// Corresponding to isbitstype() in Julia
pub unsafe fn jl_isbits<T>(t: *const T) -> bool {
    jl_is_datatype(t) && (*(t as *const jl_datatype_t)).isbitstype() != 0
}

pub unsafe fn jl_is_datatype_singleton(d: *const jl_datatype_t) -> bool {
    !(*d).instance.is_null()
}

pub unsafe fn jl_is_abstracttype<T>(v: *const T) -> bool {
    jl_is_datatype(v) && (*(*(v as *const jl_datatype_t)).name).abstract_() != 0
}

pub unsafe fn jl_is_array_type<T>(t: *const T) -> bool {
    jl_is_datatype(t) && (*(t as *const jl_datatype_t)).name == jl_array_typename
}

pub unsafe fn jl_is_array<T>(v: *const T) -> bool {
    let t = jl_typeof(v);
    jl_is_array_type(t)
}

pub unsafe fn jl_is_opaque_closure_type<T>(t: *const T) -> bool {
    jl_is_datatype(t) && (*(t as *const jl_datatype_t)).name == jl_opaque_closure_typename
}

pub unsafe fn jl_is_opaque_closure<T>(t: *const T) -> bool {
    let t = jl_typeof(t);
    jl_is_opaque_closure_type(t)
}

pub unsafe fn jl_is_cpointer_type<T>(t: *const T) -> bool {
    jl_is_datatype(t)
        && (*(t as *const jl_datatype_t)).name
            == (*((*jl_pointer_type).body as *const jl_datatype_t)).name
}

pub unsafe fn jl_is_llvmpointer_type<T>(t: *const T) -> bool {
    jl_is_datatype(t) && (*(t as *const jl_datatype_t)).name == jl_llvmpointer_typename
}

pub unsafe fn jl_is_abstract_ref_type<T>(t: *const T) -> bool {
    jl_is_datatype(t)
        && (*(t as *const jl_datatype_t)).name
            == (*((*jl_ref_type).body as *const jl_datatype_t)).name
}

pub unsafe fn jl_is_tuple_type<T>(t: *const T) -> bool {
    jl_is_datatype(t) && (*(t as *const jl_datatype_t)).name == jl_tuple_typename
}

pub unsafe fn jl_is_namedtuple_type<T>(t: *const T) -> bool {
    jl_is_datatype(t) && (*(t as *const jl_datatype_t)).name == jl_namedtuple_typename
}

pub unsafe fn jl_is_vecelement_type<T>(t: *const T) -> bool {
    jl_is_datatype(t) && (*(t as *const jl_datatype_t)).name == jl_vecelement_typename
}

pub unsafe fn jl_is_type_type<T>(v: *const T) -> bool {
    jl_is_datatype(v)
        && (*(v as *const jl_datatype_t)).name
            == (*((*jl_type_type).body as *const jl_datatype_t)).name
}

pub unsafe fn jl_is_array_zeroinit(a: *const jl_array_t) -> bool {
    if (*a).flags.ptrarray() != 0 || (*a).flags.hasptr() != 0 {
        return true;
    }
    let elty = jl_tparam0(jl_typeof(a) as *mut jl_value_t);
    jl_is_datatype(elty) && (*(elty as *const jl_datatype_t)).zeroinit() != 0
}

pub unsafe fn jl_is_dispatch_tupletype(v: *const jl_value_t) -> bool {
    jl_is_datatype(v) && (*(v as *const jl_datatype_t)).isdispatchtuple() != 0
}

pub unsafe fn jl_is_concrete_type(v: *const jl_value_t) -> bool {
    jl_is_datatype(v) && (*(v as *const jl_datatype_t)).isconcretetype() != 0
}

pub unsafe fn jl_get_function(m: *mut jl_module_t, name: *const c_char) -> *mut jl_function_t {
    jl_get_global(m, jl_symbol(name)) as *mut jl_function_t
}

pub unsafe fn jl_vinfo_sa(vi: u8) -> bool {
    (vi & 16) != 0
}

pub unsafe fn jl_vinfo_usedundef(vi: u8) -> bool {
    (vi & 32) != 0
}

pub unsafe fn jl_apply(args: *mut *mut jl_value_t, nargs: usize) -> *mut jl_value_t {
    jl_apply_generic(*args, args.add(1), (nargs - 1) as u32)
}

#[cfg(target_pointer_width = "64")]
mod box_long {
    use super::*;

    pub unsafe fn jl_box_long(x: isize) -> *mut jl_value_t {
        jl_box_int64(x as i64)
    }

    pub unsafe fn jl_box_ulong(x: usize) -> *mut jl_value_t {
        jl_box_uint64(x as u64)
    }

    pub unsafe fn jl_unbox_long(x: *mut jl_value_t) -> isize {
        jl_unbox_int64(x) as isize
    }

    pub unsafe fn jl_unbox_ulong(x: *mut jl_value_t) -> usize {
        jl_unbox_uint64(x) as usize
    }

    pub unsafe fn jl_is_long(x: *mut jl_value_t) -> bool {
        jl_is_int64(x)
    }

    pub unsafe fn jl_is_ulong(x: *mut jl_value_t) -> bool {
        jl_is_uint64(x)
    }
}

#[cfg(target_pointer_width = "32")]
mod box_long {
    use super::*;

    pub unsafe fn jl_box_long(x: isize) -> *mut jl_value_t {
        jl_box_int32(x as i32)
    }

    pub unsafe fn jl_box_ulong(x: usize) -> *mut jl_value_t {
        jl_box_uint32(x as u32)
    }

    pub unsafe fn jl_unbox_long(x: *mut jl_value_t) -> isize {
        jl_unbox_int32(x) as isize
    }

    pub unsafe fn jl_unbox_ulong(x: *mut jl_value_t) -> usize {
        jl_unbox_uint32(x) as usize
    }

    pub unsafe fn jl_is_long(x: *mut jl_value_t) -> bool {
        jl_is_int32(x)
    }

    pub unsafe fn jl_is_ulong(x: *mut jl_value_t) -> bool {
        jl_is_uint32(x)
    }
}

pub use box_long::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanity() {
        unsafe {
            jl_init();
            assert!(jl_is_initialized() != 0);

            assert!(jl_exception_occurred().is_null());

            jl_atexit_hook(0);
        }
    }
}
