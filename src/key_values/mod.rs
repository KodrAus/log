//! Structured properties for log records.

#[macro_use]
mod macros;

use std::fmt;
use std::borrow::Borrow;
use std::marker::PhantomData;
use std::cmp::Ordering;

/// A visitor for key value pairs.
/// 
/// The lifetime of the keys and values is captured by the `'kvs` type.
pub trait SourceVisitor<'kvs> {
    /// Visit a key value pair.
    fn visit_pair(&mut self, k: &'kvs dyn Key, v: &'kvs dyn Value) -> Result<(), Error>;
}

impl<'a, 'kvs, T: ?Sized> SourceVisitor<'kvs> for &'a mut T
where
    T: SourceVisitor<'kvs>,
{
    fn visit_pair(&mut self, k: &'kvs dyn Key, v: &'kvs dyn Value) -> Result<(), Error> {
        (*self).visit_pair(k, v)
    }
}

/// A source for key value pairs that can be serialized.
pub trait Source {
    /// Serialize the key value pairs.
    fn visit<'kvs>(&'kvs self, visitor: &mut dyn SourceVisitor<'kvs>) -> Result<(), Error>;

    /// Erase this `Source` so it can be used without
    /// requiring generic type parameters.
    fn erase(&self) -> ErasedSource
    where
        Self: Sized,
    {
        ErasedSource::erased(self)
    }

    /// Find the value for a given key.
    /// 
    /// If the key is present multiple times, this method will
    /// return the *last* value for the given key.
    /// 
    /// The default implementation will scan all key-value pairs.
    /// Implementors are encouraged provide a more efficient version
    /// if they can. Standard collections like `BTreeMap` and `HashMap`
    /// will do an indexed lookup instead of a scan.
    fn get<'kvs, Q>(&'kvs self, key: Q) -> Option<&'kvs dyn Value>
    where
        Q: Key,
    {
        struct Get<'k, 'v>(&'k dyn Key, Option<&'v dyn Value>);

        impl<'k, 'kvs> SourceVisitor<'kvs> for Get<'k, 'kvs> {
            fn visit_pair(&mut self, k: &'kvs dyn Key, v: &'kvs dyn Value) -> Result<(), Error> {
                if k == self.0 {
                    self.1 = Some(v);
                }

                Ok(())
            }
        }

        let mut visitor = Get(&key, None);
        let _ = self.visit(&mut visitor);

        visitor.1
    }

    /// An adapter to borrow self.
    fn by_ref(&self) -> &Self {
        self
    }

    /// Chain two `Source`s together.
    fn chain<KVS>(self, other: KVS) -> Chained<Self, KVS>
    where
        Self: Sized,
    {
        Chained(self, other)
    }

    /// Apply a function to each key-value pair.
    fn try_for_each<F, E>(self, f: F) -> Result<(), Error>
    where
        Self: Sized,
        F: FnMut(&dyn Key, &dyn Value) -> Result<(), E>,
        E: Into<Error>,
    {
        struct ForEach<F, E>(F, PhantomData<E>);

        impl<'kvs, F, E> SourceVisitor<'kvs> for ForEach<F, E>
        where
            F: FnMut(&dyn Key, &dyn Value) -> Result<(), E>,
            E: Into<Error>,
        {
            fn visit_pair(&mut self, k: &'kvs dyn Key, v: &'kvs dyn Value) -> Result<(), Error> {
                (self.0)(k, v).map_err(Into::into)
            }
        }

        self.visit(&mut ForEach(f, Default::default()))
    }

    /// Serialize the key-value pairs as a map.
    #[cfg(feature = "structured_serde")]
    fn serialize_as_map(self) -> SerializeAsMap<Self>
    where
        Self: Sized,
    {
        SerializeAsMap(self)
    }
}

impl<'a, T: ?Sized> Source for &'a T
where
    T: Source,
{
    fn visit<'kvs>(&'kvs self, visitor: &mut dyn SourceVisitor<'kvs>) -> Result<(), Error> {
        (*self).visit(visitor)
    }

    fn get<'kvs, Q>(&'kvs self, key: Q) -> Option<&'kvs dyn Value>
    where
        Q: Key,
    {
        (*self).get(key)
    }
}

/// A chain of two `Source`s.
#[derive(Debug)]
pub struct Chained<A, B>(A, B);

impl<A, B> Source for Chained<A, B>
where
    A: Source,
    B: Source,
{
    fn visit<'kvs>(&'kvs self, visitor: &mut dyn SourceVisitor<'kvs>) -> Result<(), Error> {
        self.0.visit(visitor)?;
        self.1.visit(visitor)?;

        Ok(())
    }
}

/// Serialize the key-value pairs as a map.
#[derive(Debug)]
#[cfg(feature = "structured_serde")]
pub struct SerializeAsMap<KVS>(KVS);

impl<K, V> Source for (K, V)
where
    K: Key,
    V: Value,
{
    fn visit<'kvs>(&'kvs self, visitor: &mut dyn SourceVisitor<'kvs>) -> Result<(), Error>
    {
        visitor.visit_pair(&self.0, &self.1)
    }
}

impl<KVS> Source for [KVS] where KVS: Source {
    fn visit<'kvs>(&'kvs self, visitor: &mut dyn SourceVisitor<'kvs>) -> Result<(), Error> {
        for kv in self {
            kv.visit(visitor)?;
        }

        Ok(())
    }
}

/// A key value source on a `Record`.
#[derive(Clone)]
pub struct ErasedSource<'a>(&'a dyn ErasedSourceBridge);

impl<'a> ErasedSource<'a> {
    pub fn erased(kvs: &'a impl Source) -> Self {
        ErasedSource(kvs)
    }

    pub fn empty() -> Self {
        ErasedSource(&(&[] as &[(&str, &dyn Value)]))
    }
}

impl<'a> fmt::Debug for ErasedSource<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Source").finish()
    }
}

impl<'a> Source for ErasedSource<'a> {
    fn visit<'kvs>(&'kvs self, visitor: &mut SourceVisitor<'kvs>) -> Result<(), Error> {
        self.0.erased_visit(visitor)
    }

    fn get<'kvs, Q>(&'kvs self, key: Q) -> Option<&'kvs dyn Value>
    where
        Q: Key,
    {
        self.0.erased_get(&key)
    }
}

/// A trait that erases a `Source` so it can be stored
/// in a `Record` without requiring any generic parameters.
trait ErasedSourceBridge {
    fn erased_visit<'kvs>(&'kvs self, visitor: &mut dyn SourceVisitor<'kvs>) -> Result<(), Error>;
    fn erased_get<'kvs>(&'kvs self, key: &dyn Key) -> Option<&'kvs dyn Value>;
}

impl<KVS> ErasedSourceBridge for KVS
where
    KVS: Source + ?Sized,
{
    fn erased_visit<'kvs>(&'kvs self, visitor: &mut dyn SourceVisitor<'kvs>) -> Result<(), Error> {
        self.visit(visitor)
    }

    fn erased_get<'kvs>(&'kvs self, key: &dyn Key) -> Option<&'kvs dyn Value> {
        self.get(key)
    }
}

/// An error encountered while visiting a key-value source.
pub struct Error(ErrorInner);

impl Error {
    /// Capture a static message as an error.
    pub fn msg(msg: &'static str) -> Self {
        Error(ErrorInner::Static(msg))
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

enum ErrorInner {
    Static(&'static str),
    #[cfg(feature = "std")]
    Owned(String),
}

impl fmt::Debug for ErrorInner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorInner::Static(msg) => msg.fmt(f),
            #[cfg(feature = "std")]
            ErrorInner::Owned(ref msg) => msg.fmt(f),
        }
    }
}

impl fmt::Display for ErrorInner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorInner::Static(msg) => msg.fmt(f),
            #[cfg(feature = "std")]
            ErrorInner::Owned(ref msg) => msg.fmt(f),
        }
    }
}

pub trait Key {
    fn to_str(&self) -> &str;
}

impl<'a, T: ?Sized> Key for &'a T
where
    T: Key,
{
    fn to_str(&self) -> &str {
        (**self).to_str()
    }
}

impl<'a> PartialEq for &'a dyn Key {
    fn eq(&self, other: &Self) -> bool {
        self.to_str().eq(other.to_str())
    }
}

impl<'a> Eq for &'a dyn Key { }

impl<'a> PartialOrd for &'a dyn Key {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.to_str().partial_cmp(other.to_str())
    }
}

impl<'a> Ord for &'a dyn Key {
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_str().cmp(other.to_str())
    }
}

impl<'a> fmt::Debug for &'a dyn Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.to_str().fmt(f)
    }
}

impl<'a> fmt::Display for &'a dyn Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.to_str().fmt(f)
    }
}

impl Key for str {
    fn to_str(&self) -> &str {
        self
    }
}

/// A value that can be serialized.
/// 
/// **This trait cannot be implemented manually**
/// 
/// The `Visit` trait is always implemented for a fixed set of primitives:
/// 
/// - Primitives: `bool`, `char`
/// - Unsigned integers: `u8`, `u16`, `u32`, `u64`, `u128`
/// - Signed integers: `i8`, `i16`, `i32`, `i64`, `i128`
/// - Strings: `&str`, `String`
/// - Bytes: `&[u8]`, `Vec<u8>`
/// 
/// Any other type that implements `serde::Serialize + std::fmt::Debug` will
/// automatically implement `Visit` if the `structured_serde` feature is
/// enabled.
pub trait Value: visit_imp::ValuePrivate {
    /// Visit the value with the given serializer.
    fn visit(&self, visitor: &mut dyn ValueVisitor) -> Result<(), Error>;
}

/// A serializer for primitive values.
pub trait ValueVisitor {
    /// Visit a signed integer.
    fn visit_i64(&mut self, v: i64) -> Result<(), Error> {
        self.visit_fmt(&format_args!("{:?}", v))
    }

    /// Visit an unsigned integer.
    fn visit_u64(&mut self, v: u64) -> Result<(), Error> {
        self.visit_fmt(&format_args!("{:?}", v))
    }

    /// Visit a floating point number.
    fn visit_f64(&mut self, v: f64) -> Result<(), Error> {
        self.visit_fmt(&format_args!("{:?}", v))
    }

    /// Visit a boolean.
    fn visit_bool(&mut self, v: bool) -> Result<(), Error> {
        self.visit_fmt(&format_args!("{:?}", v))
    }

    /// Visit a single character.
    fn visit_char(&mut self, v: char) -> Result<(), Error> {
        let mut b = [0; 4];
        self.visit_str(&*v.encode_utf8(&mut b))
    }

    /// Visit a UTF8 string.
    fn visit_str(&mut self, v: &str) -> Result<(), Error> {
        self.visit_fmt(&format_args!("{:?}", v))
    }

    /// Visit a raw byte buffer.
    fn visit_bytes(&mut self, v: &[u8]) -> Result<(), Error> {
        self.visit_fmt(&format_args!("{:?}", v))
    }

    /// Visit standard arguments.
    fn visit_fmt(&mut self, args: &fmt::Arguments) -> Result<(), Error>;
}

impl<'a, T: ?Sized> ValueVisitor for &'a mut T
where
    T: ValueVisitor,
{
    fn visit_i64(&mut self, v: i64) -> Result<(), Error> {
        (**self).visit_i64(v)
    }

    fn visit_u64(&mut self, v: u64) -> Result<(), Error> {
        (**self).visit_u64(v)
    }

    fn visit_f64(&mut self, v: f64) -> Result<(), Error> {
        (**self).visit_f64(v)
    }

    fn visit_bool(&mut self, v: bool) -> Result<(), Error> {
        (**self).visit_bool(v)
    }

    fn visit_char(&mut self, v: char) -> Result<(), Error> {
        (**self).visit_char(v)
    }

    fn visit_str(&mut self, v: &str) -> Result<(), Error> {
        (**self).visit_str(v)
    }

    fn visit_bytes(&mut self, v: &[u8]) -> Result<(), Error> {
        (**self).visit_bytes(v)
    }

    fn visit_fmt(&mut self, args: &fmt::Arguments) -> Result<(), Error> {
        (**self).visit_fmt(args)
    }
}

/// This trait is a private implementation detail for testing.
/// 
/// All it does is make sure that our set of concrete types
/// that implement `Visit` always implement the `Visit` trait,
/// regardless of crate features and blanket implementations.
trait EnsureValue: Value {}

macro_rules! ensure_impl_visit {
    ($(<$($params:tt),*> $ty:ty { $($serialize:tt)* })*) => {
        $(
            impl<$($params),*> EnsureValue for $ty {}
            impl<'a, $($params),*> EnsureValue for &'a $ty {}

            #[cfg(not(feature = "structured_serde"))]
            impl<$($params),*> Value for $ty {
                $($serialize)*
            }

            #[cfg(not(feature = "structured_serde"))]
            impl<$($params),*> visit_imp::ValuePrivate for $ty {}
        )*
    };
    ($($ty:ty { $($serialize:tt)* })*) => {
        $(
            impl EnsureValue for $ty {}
            impl<'a> EnsureValue for &'a $ty {}

            #[cfg(not(feature = "structured_serde"))]
            impl Value for $ty {
                $($serialize)*
            }

            #[cfg(not(feature = "structured_serde"))]
            impl visit_imp::ValuePrivate for $ty {}
        )*
    }
}

ensure_impl_visit! {
    u8 {
        fn visit(&self, visitor: &mut dyn ValueVisitor) -> Result<(), Error> {
            visitor.visit_u64(*self as u64)
        }
    }
    u16 {
        fn visit(&self, visitor: &mut dyn ValueVisitor) -> Result<(), Error> {
            visitor.visit_u64(*self as u64)
        }
    }
    u32 {
        fn visit(&self, visitor: &mut dyn ValueVisitor) -> Result<(), Error> {
            visitor.visit_u64(*self as u64)
        }
    }
    u64 {
        fn visit(&self, visitor: &mut dyn ValueVisitor) -> Result<(), Error> {
            visitor.visit_u64(*self)
        }
    }

    i8 {
        fn visit(&self, visitor: &mut dyn ValueVisitor) -> Result<(), Error> {
            visitor.visit_i64(*self as i64)
        }
    }
    i16 {
        fn visit(&self, visitor: &mut dyn ValueVisitor) -> Result<(), Error> {
            visitor.visit_i64(*self as i64)
        }
    }
    i32 {
        fn visit(&self, visitor: &mut dyn ValueVisitor) -> Result<(), Error> {
            visitor.visit_i64(*self as i64)
        }
    }
    i64 {
        fn visit(&self, visitor: &mut dyn ValueVisitor) -> Result<(), Error> {
            visitor.visit_i64(*self)
        }
    }

    f32 {
        fn visit(&self, visitor: &mut dyn ValueVisitor) -> Result<(), Error> {
            visitor.visit_f64(*self as f64)
        }
    }
    f64 {
        fn visit(&self, visitor: &mut dyn ValueVisitor) -> Result<(), Error> {
            visitor.visit_f64(*self)
        }
    }

    char {
        fn visit(&self, visitor: &mut dyn ValueVisitor) -> Result<(), Error> {
            visitor.visit_char(*self)
        }
    }
    bool {
        fn visit(&self, visitor: &mut dyn ValueVisitor) -> Result<(), Error> {
            visitor.visit_bool(*self)
        }
    }
    str {
        fn visit(&self, visitor: &mut dyn ValueVisitor) -> Result<(), Error> {
            visitor.visit_str(self)
        }
    }
    [u8] {
        fn visit(&self, visitor: &mut dyn ValueVisitor) -> Result<(), Error> {
            visitor.visit_bytes(self)
        }
    }
}

#[cfg(not(feature = "structured_serde"))]
mod visit_imp {
    use super::*;

    #[doc(hidden)]
    pub trait ValuePrivate: fmt::Debug {}

    impl<'a, T: ?Sized> Value for &'a T
    where
        T: Value,
    {
        fn visit(&self, visitor: &mut dyn ValueVisitor) -> Result<(), Error> {
            (**self).visit(visitor)
        }
    }

    impl<'a, T: ?Sized> ValuePrivate for &'a T
    where
        T: Value,
    {
    }
}

#[cfg(feature = "structured_serde")]
mod visit_imp {
    use super::*;

    use erased_serde;
    use serde::{Serialize, Serializer};

    #[doc(hidden)]
    pub trait ValuePrivate: erased_serde::Serialize + fmt::Debug {}
 
    impl<T: ?Sized> Value for T
    where
        T: Serialize + fmt::Debug,
    {
        fn visit(&self, visitor: &mut dyn ValueVisitor) -> Result<(), Error> {
            match Serialize::serialize(self, SerdeBridge(visitor)) {
                Err(SerdeError::Unsupported) => visitor.visit_fmt(&format_args!("{:?}", self)),
                Err(SerdeError::Other(e)) => Err(e),
                Ok(()) => Ok(())
            }
        }
    }

    impl<T: ?Sized> ValuePrivate for T
    where
        T: Serialize + fmt::Debug,
    {
    }

    struct SerdeBridge<'a>(&'a mut dyn ValueVisitor);

    #[derive(Debug)]
    enum SerdeError {
        Unsupported,
        Other(Error),
    }

    impl serde::ser::Error for SerdeError {
        fn custom<T>(_msg: T) -> Self
        where
            T: std::fmt::Display
        {
            SerdeError::Unsupported
        }
    }

    impl std::fmt::Display for SerdeError {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "serialization error")
        }
    }

    impl std::error::Error for SerdeError {
        fn cause(&self) -> Option<&dyn std::error::Error> {
            None
        }

        fn description(&self) -> &str {
            "serialization error"
        }
    }

    impl<'a> Serializer for SerdeBridge<'a> {
        type Ok = ();
        type Error = SerdeError;

        type SerializeSeq = serde::ser::Impossible<Self::Ok, Self::Error>;
        type SerializeTuple = serde::ser::Impossible<Self::Ok, Self::Error>;
        type SerializeTupleStruct = serde::ser::Impossible<Self::Ok, Self::Error>;
        type SerializeTupleVariant = serde::ser::Impossible<Self::Ok, Self::Error>;
        type SerializeMap = serde::ser::Impossible<Self::Ok, Self::Error>;
        type SerializeStruct = serde::ser::Impossible<Self::Ok, Self::Error>;
        type SerializeStructVariant = serde::ser::Impossible<Self::Ok, Self::Error>;

        fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
            self.0.visit_bool(v).map_err(SerdeError::Other)
        }

        fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
            self.serialize_i64(v as i64)
        }

        fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
            self.serialize_i64(v as i64)
        }

        fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
            self.serialize_i64(v as i64)
        }

        fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
            self.0.visit_i64(v).map_err(SerdeError::Other)
        }

        fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
            self.serialize_u64(v as u64)
        }

        fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
            self.serialize_u64(v as u64)
        }

        fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
            self.serialize_u64(v as u64)
        }

        fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
            self.0.visit_u64(v).map_err(SerdeError::Other)
        }

        fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
            self.serialize_f64(v as f64)
        }

        fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
            self.0.visit_f64(v).map_err(SerdeError::Other)
        }

        fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
            self.0.visit_char(v).map_err(SerdeError::Other)
        }

        fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
            self.0.visit_str(v).map_err(SerdeError::Other)
        }

        fn collect_str<T: std::fmt::Display + ?Sized>(self, v: &T) -> Result<Self::Ok, Self::Error> {
            self.0.visit_fmt(&format_args!("{}", v)).map_err(SerdeError::Other)
        }

        fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
            self.0.visit_bytes(v).map_err(SerdeError::Other)
        }

        fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
            Err(SerdeError::Unsupported)
        }

        fn serialize_some<T>(self, v: &T) -> Result<Self::Ok, Self::Error>
        where
            T: ?Sized + Serialize,
        {
            v.serialize(self)
        }

        fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
            Err(SerdeError::Unsupported)
        }

        fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
            Err(SerdeError::Unsupported)
        }

        fn serialize_unit_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            _variant: &'static str,
        ) -> Result<Self::Ok, Self::Error> {
            Err(SerdeError::Unsupported)
        }

        fn serialize_newtype_struct<T>(
            self,
            _name: &'static str,
            _value: &T,
        ) -> Result<Self::Ok, Self::Error>
        where
            T: ?Sized + Serialize,
        {
            Err(SerdeError::Unsupported)
        }

        fn serialize_newtype_variant<T>(
            self,
            _name: &'static str,
            _variant_index: u32,
            _variant: &'static str,
            _value: &T,
        ) -> Result<Self::Ok, Self::Error>
        where
            T: ?Sized + Serialize,
        {
            Err(SerdeError::Unsupported)
        }

        fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
            Err(SerdeError::Unsupported)
        }

        fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
            Err(SerdeError::Unsupported)
        }

        fn serialize_tuple_struct(
            self,
            _name: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeTupleStruct, Self::Error> {
            Err(SerdeError::Unsupported)
        }

        fn serialize_tuple_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            _variant: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeTupleVariant, Self::Error> {
            Err(SerdeError::Unsupported)
        }

        fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
            Err(SerdeError::Unsupported)
        }

        fn serialize_struct(
            self,
            _name: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeStruct, Self::Error> {
            Err(SerdeError::Unsupported)
        }

        fn serialize_struct_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            _variant: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeStructVariant, Self::Error> {
            Err(SerdeError::Unsupported)
        }
    }
}

#[cfg(feature = "structured_serde")]
mod serde_support {
    use super::*;

    use serde::{self, Serialize, Serializer};

    impl<KVS> Serialize for SerializeAsMap<KVS>
    where
        KVS: Source,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            use serde::ser::SerializeMap;

            let mut map = serializer.serialize_map(None)?;

            self.0
                .by_ref()
                .try_for_each(|k, v| map.serialize_entry(&k, &v))
                .map_err(Error::into_serde)?;

            map.end()
        }
    }

    impl<'a> Serialize for dyn Key + 'a {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            self.to_str().serialize(serializer)
        }
    }

    impl<'a> Serialize for dyn Value + 'a {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            erased_serde::serialize(self, serializer)
        }
    }

    impl Error {
        pub fn into_serde<E>(self) -> E
        where
            E: serde::ser::Error,
        {
            E::custom(self)
        }
    }
}

#[cfg(feature = "structured_serde")]
pub use self::serde_support::*;

#[cfg(feature = "std")]
mod std_support {
    use super::*;

    use std::error;
    use std::hash::Hash;
    use std::collections::{HashMap, BTreeMap};

    impl<KVS> Source for Vec<KVS> where KVS: Source {
        fn visit<'kvs>(&'kvs self, visitor: &mut dyn SourceVisitor<'kvs>) -> Result<(), Error> {
            self.as_slice().visit(visitor)
        }
    }

    impl<K, V> Source for BTreeMap<K, V>
    where
        K: Key + Borrow<str> + Ord,
        V: Value,
    {
        fn visit<'kvs>(&'kvs self, visitor: &mut dyn SourceVisitor<'kvs>) -> Result<(), Error>
        {
            for (k, v) in self {
                visitor.visit_pair(&*k, &*v)?;
            }

            Ok(())
        }

        fn get<'kvs, Q>(&'kvs self, key: Q) -> Option<&'kvs dyn Value>
        where
            Q: Key,
        {
            BTreeMap::get(self, key.to_str()).map(|v| v as &dyn Value)
        }
    }

    impl<K, V> Source for HashMap<K, V>
    where
        K: Key + Borrow<str> + Eq + Hash,
        V: Value,
    {
        fn visit<'kvs>(&'kvs self, visitor: &mut dyn SourceVisitor<'kvs>) -> Result<(), Error>
        {
            for (k, v) in self {
                visitor.visit_pair(&*k, &*v)?;
            }

            Ok(())
        }

        fn get<'kvs, Q>(&'kvs self, key: Q) -> Option<&'kvs dyn Value>
        where
            Q: Key,
        {
            HashMap::get(self, key.to_str()).map(|v| v as &dyn Value)
        }
    }

    ensure_impl_visit! {
        String {
            fn visit(&self, visitor: &mut dyn ValueVisitor) -> Result<(), Error> {
                visitor.visit_str(&*self)
            }
        }
        Vec<u8> {
            fn visit(&self, visitor: &mut dyn ValueVisitor) -> Result<(), Error> {
                visitor.visit_bytes(&*self)
            }
        }
    }

    impl Error {
        pub fn as_error(&self) -> &(dyn error::Error + Send + Sync + 'static) {
            &self.0
        }

        pub fn into_error(self) -> Box<dyn error::Error + Send + Sync> {
            Box::new(self.0)
        }
    }

    impl<E> From<E> for Error
    where
        E: error::Error,
    {
        fn from(err: E) -> Self {
            Error(ErrorInner::Owned(err.to_string()))
        }
    }

    impl AsRef<dyn error::Error + Send + Sync + 'static> for Error {
        fn as_ref(&self) -> &(dyn error::Error + Send + Sync + 'static) {
            self.as_error()
        }
    }

    impl From<Error> for Box<dyn error::Error + Send + Sync> {
        fn from(err: Error) -> Self {
            err.into_error()
        }
    }

    impl error::Error for ErrorInner {
        fn description(&self) -> &str {
            match self {
                ErrorInner::Static(msg) => msg,
                ErrorInner::Owned(msg) => msg,
            }
        }
    }
}

#[cfg(feature = "std")]
pub use self::std_support::*;

#[cfg(test)]
mod tests {
    use crate::*;

    #[derive(PartialEq, Debug)]
    enum Token<'a> {
        I64(i64),
        U64(u64),
        F64(f64),
        Bool(bool),
        Char(char),
        Str(&'a str),
        Bytes(&'a [u8]),
        Args(&'a str),
    }

    // `&dyn ser::Serialize` should impl `Serialize`
    fn assert_visit(v: &dyn Value, token: Token) {
        struct TestVisitor<'a>(Token<'a>);

        impl<'a> ValueVisitor for TestVisitor<'a> {
            fn visit_i64(&mut self, v: i64) {
                assert_eq!(self.0, Token::I64(v));
            }
            
            fn visit_u64(&mut self, v: u64) {
                assert_eq!(self.0, Token::U64(v));
            }

            fn visit_f64(&mut self, v: f64) {
                assert_eq!(self.0, Token::F64(v));
            }

            fn visit_bool(&mut self, v: bool) {
                assert_eq!(self.0, Token::Bool(v));
            }

            fn visit_char(&mut self, v: char) {
                assert_eq!(self.0, Token::Char(v));
            }

            fn visit_str(&mut self, v: &str) {
                assert_eq!(self.0, Token::Str(v));
            }

            fn visit_bytes(&mut self, v: &[u8]) {
                assert_eq!(self.0, Token::Bytes(v));
            }

            fn visit_fmt(&mut self, v: &fmt::Arguments) {
                use self::std::{str, ptr};
                use self::fmt::Write;

                const LEN: usize = 128;

                struct ValueArgs {
                    buf: [u8; LEN],
                    cursor: usize,
                }

                impl ValueArgs {
                    fn new() -> Self {
                        ValueArgs {
                            buf: [0; LEN],
                            cursor: 0,
                        }
                    }

                    fn to_str(&self) -> Option<&str> {
                        str::from_utf8(&self.buf[0..self.cursor]).ok()
                    }
                }

                impl Write for ValueArgs {
                    fn write_str(&mut self, s: &str) -> fmt::Result {
                        let src = s.as_bytes();
                        let next_cursor = self.cursor + src.len();

                        if next_cursor > LEN {
                            return Err(fmt::Error);
                        }

                        unsafe {
                            let src_ptr = src.as_ptr();
                            let dst_ptr = self.buf.as_mut_ptr().offset(self.cursor as isize);

                            ptr::copy_nonoverlapping(src_ptr, dst_ptr, src.len());
                        }

                        self.cursor = next_cursor;

                        Ok(())
                    }
                }

                let mut w = ValueArgs::new();
                w.write_fmt(format_args!("{}", v)).unwrap();
                assert_eq!(self.0, Token::Args(w.to_str().unwrap()));
            }
        }

        v.visit(&mut TestVisitor(token));
    }

    #[test]
    fn visit_simple() {
        assert_visit(&1u8, Token::U64(1u64));
        assert_visit(&true, Token::Bool(true));
        assert_visit(&"a string", Token::Str("a string"));
    }

    #[test]
    #[cfg(feature = "structured_serde")]
    fn visit_unsupported_as_debug() {
        use serde_json::json;

        let v = json!({
            "id": 123,
            "name": "alice",
        });

        assert_visit(&v, Token::Args(&format!("{:?}", v)));
    }

    #[cfg(feature = "structured_serde")]
    mod structured_serde {
        use crate::*;
        use serde_test::{Token, assert_ser_tokens};
        use serde_json::json;

        // `&dyn ser::Serialize` should impl `Serialize`
        fn assert_visit(v: &dyn Value, tokens: &[Token]) {
            assert_ser_tokens(&v, tokens);
        }

        #[test]
        fn visit_simple() {
            assert_visit(&1u8, &[Token::U8(1u8)]);
            assert_visit(&true, &[Token::Bool(true)]);
            assert_visit(&"a string", &[Token::Str("a string")]);
        }

        #[test]
        fn visit_complex() {
            let v = json!({
                "id": 123,
                "name": "alice",
            });

            assert_visit(&v, &[
                Token::Map { len: Some(2) },
                Token::Str("id"),
                Token::U64(123),
                Token::Str("name"),
                Token::Str("alice"),
                Token::MapEnd,
            ]);
        }
    }
}
