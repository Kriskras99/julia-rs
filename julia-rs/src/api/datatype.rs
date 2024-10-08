//! Module providing wrappers for the native Julia type-types.

use std::convert::TryFrom;
use std::ptr;
use std::result;

use crate::api::{Array, IntoSymbol, JlValue, Svec, Value};
use crate::error::{Error, Result};
use crate::jlvalues;
use crate::sys::*;

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum VarargKind {
    None,
    Int,
    Bound,
    Unbound,
}

impl TryFrom<u32> for VarargKind {
    type Error = ();
    fn try_from(kind: u32) -> result::Result<Self, ()> {
        match kind {
            0 => Ok(Self::None),
            1 => Ok(Self::Int),
            2 => Ok(Self::Bound),
            3 => Ok(Self::Unbound),
            _ => Err(()),
        }
    }
}

jlvalues! {
    pub struct Type(jl_value_t);
    pub struct Datatype(jl_datatype_t);
    pub struct Union(jl_uniontype_t);
    pub struct UnionAll(jl_unionall_t);
    pub struct Tuple(jl_tupletype_t);
}

impl Type {
    /// Creates a new Julia array of this type.
    pub fn new_array<I>(&self, params: I) -> Result<Array>
    where
        I: IntoIterator<Item = Value>,
    {
        let mut paramv = vec![];
        for p in params {
            paramv.push(p.lock()?);
        }

        let dt = self.lock()?;
        let array = unsafe { jl_alloc_array_1d(dt as *mut _, paramv.len()) };
        jl_catch!();

        for (i, p) in paramv.into_iter().enumerate() {
            unsafe {
                jl_arrayset(array, p, i);
            }
        }
        jl_catch!();

        Array::new(array)
    }

    pub fn apply_type<'a, I>(&self, params: I) -> Result<Self>
    where
        I: IntoIterator<Item = &'a Value>,
    {
        let mut paramv = vec![];
        for p in params {
            paramv.push(p.lock()?);
        }
        let nparam = paramv.len();
        let paramv = paramv.as_mut_ptr();

        let tc = self.lock()?;
        let raw = unsafe { jl_apply_type(tc, paramv, nparam) };
        jl_catch!();
        Self::new(raw)
    }

    pub fn apply_type1(&self, p1: &Value) -> Result<Self> {
        let tc = self.lock()?;
        let p1 = p1.lock()?;

        let raw = unsafe { jl_apply_type1(tc, p1) };
        jl_catch!();
        Self::new(raw)
    }

    pub fn apply_type2(&self, p1: &Value, p2: &Value) -> Result<Self> {
        let tc = self.lock()?;
        let p1 = p1.lock()?;
        let p2 = p2.lock()?;

        let raw = unsafe { jl_apply_type2(tc, p1, p2) };
        jl_catch!();
        Self::new(raw)
    }

    /// Applies function to the inner pointer.
    pub fn map<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(*mut jl_value_t) -> T,
    {
        self.lock().map(f)
    }

    /// Applies function to the inner pointer and returns a default value if
    /// its poisoned.
    pub fn map_or<T, F>(&self, f: F, optb: T) -> T
    where
        F: FnOnce(*mut jl_value_t) -> T,
    {
        self.lock().map(f).unwrap_or(optb)
    }

    /// Applies function to the inner pointer and executes a default function if
    /// its poisoned.
    pub fn map_or_else<T, F, O>(&self, f: F, op: O) -> T
    where
        F: FnOnce(*mut jl_value_t) -> T,
        O: FnOnce(Error) -> T,
    {
        self.lock().map(f).unwrap_or_else(op)
    }

    /// Checks if the inner Mutex is poisoned.
    pub fn is_ok(&self) -> bool {
        !self._inner.is_poisoned()
    }

    /// Checks if the value is a type.
    pub fn is_type(&self) -> bool {
        self.map_or(|v| unsafe { jl_is_type(v as *mut _) }, false)
    }
    /// Checks if the value is a kind.
    pub fn is_kind(&self) -> bool {
        self.map_or(|v| unsafe { jl_is_kind(v as *mut _) }, false)
    }
    /// Checks if the value is a primitivetype.
    pub fn is_primitivetype(&self) -> bool {
        self.map_or(|v| unsafe { jl_is_primitivetype(v) }, false)
    }
    /// Checks if the value is a structtype.
    pub fn is_structtype(&self) -> bool {
        self.map_or(|v| unsafe { jl_is_structtype(v) }, false)
    }
    /// Checks if the value is a array_type.
    pub fn is_array_type(&self) -> bool {
        self.map_or(|v| unsafe { jl_is_array_type(v) }, false)
    }
    /// Checks if the value is a abstracttype.
    pub fn is_abstracttype(&self) -> bool {
        self.map_or(|v| unsafe { jl_is_abstracttype(v) }, false)
    }
    /// Checks if the value is a array.
    pub fn is_array(&self) -> bool {
        self.map_or(|v| unsafe { jl_is_array(v) }, false)
    }
    /// Checks if the value is a cpointer_type.
    pub fn is_cpointer_type(&self) -> bool {
        self.map_or(|v| unsafe { jl_is_cpointer_type(v) }, false)
    }
    /// Checks if the value is a abstract_ref_type.
    pub fn is_abstract_ref_type(&self) -> bool {
        self.map_or(|v| unsafe { jl_is_abstract_ref_type(v) }, false)
    }
    /// Checks if the value is a tuple_type.
    pub fn is_tuple_type(&self) -> bool {
        self.map_or(|v| unsafe { jl_is_tuple_type(v) }, false)
    }
    /// Checks if the value is a vecelement_type.
    pub fn is_vecelement_type(&self) -> bool {
        self.map_or(|v| unsafe { jl_is_vecelement_type(v) }, false)
    }
    /// Checks if the value is a type_type.
    pub fn is_type_type(&self) -> bool {
        self.map_or(|v| unsafe { jl_is_type_type(v) }, false)
    }
}

impl Datatype {
    /// Creates a new Julia struct of this type.
    pub fn new_struct<'a, I>(&self, params: I) -> Result<Value>
    where
        I: IntoIterator<Item = &'a Value>,
    {
        let mut paramv = vec![];
        for p in params {
            paramv.push(p.lock()?);
        }
        let nparam = paramv.len();
        let paramv = paramv.as_mut_ptr();

        let dt = self.lock()?;
        let value = unsafe { jl_new_structv(dt, paramv, nparam as u32) };
        jl_catch!();
        Value::new(value)
    }

    /// Creates a new Julia primitive of this type.
    pub fn new_bits<T: Into<Vec<u8>>>(&self, data: T) -> Result<Value> {
        let data = data.into();
        let bits = data.as_ptr();

        let dt = self.lock()?;
        let value = unsafe { jl_new_bits(dt as *mut _, bits as *mut _) };
        jl_catch!();
        Value::new(value)
    }

    pub fn any() -> Self {
        unsafe { Self::new_unchecked(jl_any_type) }
    }
    pub fn number() -> Self {
        unsafe { Self::new_unchecked(jl_number_type) }
    }
    pub fn signed() -> Self {
        unsafe { Self::new_unchecked(jl_signed_type) }
    }
    pub fn abstract_float() -> Self {
        unsafe { Self::new_unchecked(jl_floatingpoint_type) }
    }
    pub fn bool() -> Self {
        unsafe { Self::new_unchecked(jl_bool_type) }
    }
    pub fn char() -> Self {
        unsafe { Self::new_unchecked(jl_char_type) }
    }
    pub fn int8() -> Self {
        unsafe { Self::new_unchecked(jl_int8_type) }
    }
    pub fn uint8() -> Self {
        unsafe { Self::new_unchecked(jl_uint8_type) }
    }
    pub fn int16() -> Self {
        unsafe { Self::new_unchecked(jl_int16_type) }
    }
    pub fn uint16() -> Self {
        unsafe { Self::new_unchecked(jl_uint16_type) }
    }
    pub fn int32() -> Self {
        unsafe { Self::new_unchecked(jl_int32_type) }
    }
    pub fn uint32() -> Self {
        unsafe { Self::new_unchecked(jl_uint32_type) }
    }
    pub fn int64() -> Self {
        unsafe { Self::new_unchecked(jl_int64_type) }
    }
    pub fn uint64() -> Self {
        unsafe { Self::new_unchecked(jl_uint64_type) }
    }
    pub fn float16() -> Self {
        unsafe { Self::new_unchecked(jl_float16_type) }
    }
    pub fn float32() -> Self {
        unsafe { Self::new_unchecked(jl_float32_type) }
    }
    pub fn float64() -> Self {
        unsafe { Self::new_unchecked(jl_float64_type) }
    }
    pub fn void() -> Self {
        unsafe { Self::new_unchecked(jl_void_type) }
    }
    pub fn void_pointer() -> Self {
        unsafe { Self::new_unchecked(jl_voidpointer_type) }
    }
    pub fn pointer() -> Self {
        unsafe { Self::new_unchecked(jl_pointer_type as *mut _) }
    }
}

impl Default for Datatype {
    fn default() -> Self {
        Self::any()
    }
}

impl Union {
    /// Create a union of types.
    #[allow(clippy::self_named_constructors)]
    pub fn union<'a, I>(ts: I) -> Result<Self>
    where
        I: IntoIterator<Item = &'a Datatype>,
    {
        let mut vec = vec![];
        for t in ts {
            vec.push(t.lock()?);
        }
        let n = vec.len();
        let ts_ptr = vec.as_mut_ptr();

        let raw = unsafe { jl_type_union(ts_ptr as *mut *mut _, n) };
        jl_catch!();
        Self::new(raw as *mut _)
    }

    /// Get the union that is an intersection of two types.
    pub fn intersection(a: &Self, b: &Self) -> Result<Self> {
        let a = a.lock()?;
        let b = b.lock()?;

        let raw = unsafe { jl_type_intersection(a as *mut _, b as *mut _) };
        jl_catch!();
        Self::new(raw as *mut _)
    }

    /// Check if the intersection of two unions is empty.
    pub fn has_empty_intersection(a: &Self, b: &Self) -> Result<bool> {
        let a = a.lock()?;
        let b = b.lock()?;

        let p = unsafe { jl_has_empty_intersection(a as *mut _, b as *mut _) };
        jl_catch!();
        Ok(p != 0)
    }
}

impl UnionAll {
    /// Instantiate a UnionAll into a more concrete type.
    /// Not guaranteed to be a concrete datatype.
    pub fn instantiate(&self, p: &Value) -> Result<Type> {
        let inner = self.lock()?;
        let p = p.lock()?;

        let raw = unsafe { jl_instantiate_unionall(inner, p) };
        jl_catch!();
        Type::new(raw)
    }
}

impl Tuple {
    pub fn apply(params: &Svec) -> Result<Self> {
        let params = params.lock()?;

        let raw = unsafe { jl_apply_tuple_type(params) };
        jl_catch!();
        Self::new(raw as *mut jl_tupletype_t)
    }
}

/// Type for constructing new primitive, abstract or compound types.
pub struct TypeBuilder {
    name: *mut jl_sym_t,
    supertype: *mut jl_datatype_t,
    params: *mut jl_svec_t,
    fnames: *mut jl_svec_t,
    ftypes: *mut jl_svec_t,
    nbits: usize,
    abstrac: bool,
    mutable: bool,
    ninitialized: bool,
    primitive: bool,
    err: Option<Error>,
}

impl TypeBuilder {
    /// Construct a new default TypeBuilder;
    pub fn new() -> Self {
        Self {
            name: ptr::null_mut(),
            supertype: unsafe { jl_any_type },
            params: unsafe { jl_emptysvec },
            fnames: unsafe { jl_emptysvec },
            ftypes: unsafe { jl_emptysvec },
            nbits: 0,
            abstrac: false,
            mutable: false,
            ninitialized: false,
            primitive: false,
            err: None,
        }
    }

    /// Get the error if it occurred.
    pub const fn err(&self) -> Option<&Error> {
        self.err.as_ref()
    }

    /// Check if any error occurred.
    pub const fn is_err(&self) -> bool {
        self.err.is_some()
    }

    // /// Builds the Type. If any errors occurred previously, they will be returned here.
    // pub fn build(self) -> Result<Datatype> {
    //     if let Some(err) = self.err {
    //         return Err(err);
    //     }

    //     if self.primitive {
    //         let raw = unsafe {
    //             jl_new_primitivetype(self.name as *mut _, self.supertype, self.params, self.nbits)
    //         };
    //         jl_catch!();
    //         Datatype::new(raw)
    //     } else {
    //         let raw = unsafe {
    //             jl_new_datatype(
    //                 self.name,
    //                 self.supertype,
    //                 self.params,
    //                 self.fnames,
    //                 self.ftypes,
    //                 self.abstrac as i32,
    //                 self.mutable as i32,
    //                 self.ninitialized as i32,
    //             )
    //         };
    //         jl_catch!();
    //         Datatype::new(raw)
    //     }
    // }

    /// Sets the name.
    pub fn name<S: IntoSymbol>(mut self, name: S) -> Self {
        let name = name.into_symbol();

        if let Err(err) = name {
            self.err = Some(err);
            return self;
        }

        let name = name.unwrap().into_inner();

        self.name = match name {
            Ok(name) => name,
            Err(err) => {
                self.err = Some(err);
                return self;
            }
        };
        self
    }

    /// Sets the supertype. Must be an abstract.
    pub fn supertype(mut self, supertype: &Datatype) -> Self {
        self.supertype = match supertype.lock() {
            Ok(supertype) => supertype,
            Err(err) => {
                self.err = Some(err);
                return self;
            }
        };
        self
    }

    pub fn params(mut self, params: &Svec) -> Self {
        self.params = match params.lock() {
            Ok(params) => params,
            Err(err) => {
                self.err = Some(err);
                return self;
            }
        };
        self
    }

    /// Sets the names of the fields.
    pub fn fnames(mut self, fnames: &Svec) -> Self {
        self.fnames = match fnames.lock() {
            Ok(fnames) => fnames,
            Err(err) => {
                self.err = Some(err);
                return self;
            }
        };
        self
    }

    /// Sets the types of the fields.
    pub fn ftypes(mut self, ftypes: &Svec) -> Self {
        self.ftypes = match ftypes.lock() {
            Ok(ftypes) => ftypes,
            Err(err) => {
                self.err = Some(err);
                return self;
            }
        };
        self
    }

    /// Sets the number of bits in a primitive. Must be a multiple of 8.
    pub const fn nbits(mut self, nbits: usize) -> Self {
        self.nbits = nbits;
        self
    }

    /// Sets whether the type is abstract.
    pub const fn abstrac(mut self, abstrac: bool) -> Self {
        self.abstrac = abstrac;
        self
    }

    /// Sets whether the struct is mutable.
    pub const fn mutable(mut self, mutable: bool) -> Self {
        self.mutable = mutable;
        self
    }

    pub const fn ninitialized(mut self, ninitialized: bool) -> Self {
        self.ninitialized = ninitialized;
        self
    }

    /// Sets whether the type is a primitive.
    pub const fn primitive(mut self, primitive: bool) -> Self {
        self.primitive = primitive;
        self
    }
}

impl Default for TypeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a new Julia type using a Rust-like syntax.
///
/// # Syntax
///
/// ## Primitive type
/// ```
/// type <name> = Bits<N> where N: <bits> [ , Self: <supertype> ];
/// ```
///
/// ## Abstract type
/// ```
/// trait <name> [ : <supertype> ];
/// ```
///
/// ## Struct
/// ```
/// [mut] struct <name> [ : <supertype> ];
/// ```
/// **or**
/// ```
/// [mut] struct <name> {
///     (
///         <fname>: <ftype>,
///     )*
/// } [ : <supertype> ]
/// ```
#[macro_export]
macro_rules! jl_type {
    { type $name:ident = Bits<N> where N: $nbits:expr; } => {
        jl_type! { type $name = Bits<N> where N: $nbits, Self : Datatype::any(); }
    };
    { type $name:ident = Bits<N> where N: $nbits:expr, Self : $supertype:expr; } => {
        TypeBuilder::new()
            .primitive(true)
            .name(stringify!($name))
            .supertype(&$supertype)
            .nbits($nbits)
            .build()
    };
    { trait $name:ident; } => {
        jl_type! { type $name: Datatype::any(); }
    };
    { trait $name:ident : $supertype:expr; } => {
        TypeBuilder::new()
            .abstrac(true)
            .name(stringify!($name))
            .supertype(&$supertype)
            .build()
    };
    { struct $name:ident; } => {
        jl_type! { struct $name: Datatype::any(); }
    };
    { struct $name:ident : $supertype:expr; } => {
        TypeBuilder::new()
            .name(stringify!($name))
            .supertype(&$supertype)
            .build()
    };
    {
        struct $name:ident {
            $(
                $fname:ident : $ftype:expr
            ),*
        }
    } => {
        jl_type! {
            struct $name {
                $(
                    $fname : $ftype,
                )*
            } : Datatype::any()
        }
    };
    {
        struct $name:ident {
            $(
                $fname:ident : $ftype:expr,
            )*
        } : $supertype:expr
    } => {
        {
            use $crate::error::Result;
            use $crate::api::{IntoSymbol, Datatype};

            fn build() -> Result<Datatype> {
                TypeBuilder::new()
                    .name(stringify!($name))
                    .supertype(&$supertype)
                    .fnames(&jlvec![
                            $(
                                Value::from_value(
                                    stringify!($fname).into_symbol()?
                                )?
                            ),*
                        ]?)
                    .ftypes(&jlvec![
                            $( Value::from_value($ftype)? ),*
                        ]?)
                    .build()
            }

            build()
        }
    };
    { mut struct $name:ident; } => {
        jl_type! { mut struct $name: Datatype::any(); }
    };
    { mut struct $name:ident : $supertype:expr; } => {
        TypeBuilder::new()
            .mutable(true)
            .name(stringify!($name))
            .supertype(&$supertype)
            .build()
    };
    {
        mut struct $name:ident {
            $(
                $fname:ident : $ftype:expr,
            )*
        }
    } => {
        jl_type! {
            mut struct $name {
            $(
                $fname : $ftype,
            )*
            } : Datatype::any()
        }
    };
    {
        mut struct $name:ident {
            $(
                $fname:ident : $ftype:expr,
            )*
        } : $supertype:expr
    } => {
        {
            use $crate::error::Result;
            use $crate::api::{IntoSymbol, Datatype};

            fn build() -> Result<Datatype> {
                TypeBuilder::new()
                    .mutable(true)
                    .name(stringify!($name))
                    .supertype(&$supertype)
                    .fnames(&jlvec![
                            $(
                                Value::from_value(
                                    stringify!($fname).into_symbol()?
                                )?
                            ),*
                        ]?)
                    .ftypes(&jlvec![
                            $( Value::from_value($ftype)? ),*
                        ]?)
                    .build()
            }

            build()
        }
    };
}
