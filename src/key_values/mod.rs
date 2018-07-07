//! Log record properties.

#[macro_use]
mod macros;
mod primitive;

pub mod adapter;

#[cfg(feature = "std")]
use std::collections;
#[cfg(feature = "std")]
use std::hash;

use std::fmt;

use serde;
#[cfg(feature = "erased-serde")]
use erased_serde;

/// A visitor for key value pairs.
pub trait Visitor {
    /// Visit the key and value.
    fn visit_kv(&mut self, kv: &dyn KeyValue);
}

/// A set of key value pairs that can be serialized.
pub trait KeyValues {
    /// Serialize the key value pairs.
    fn visit(&self, visitor: &mut dyn Visitor);
}

/// A single key value pair.
pub trait KeyValue {
    /// Get the key.
    fn key(&self) -> Key;
    /// Get the value.
    fn value(&self) -> Value;
}

/// Converting into a `Value`.
pub trait ToValue {
    /// Perform the conversion.
    fn to_value(&self) -> Value;
}

/// Converting into a `Key`.
pub trait ToKey {
    /// Perform the conversion.
    fn to_key(&self) -> Key;
}

impl<KV> KeyValues for [KV] where KV: KeyValue {
    fn visit(&self, visitor: &mut dyn Visitor) {
        for kv in self {
            visitor.visit_kv(&kv);
        }
    }
}

#[cfg(feature = "std")]
impl<KV> KeyValues for Vec<KV> where KV: KeyValue {
    fn visit(&self, visitor: &mut dyn Visitor) {
        self.as_slice().visit(visitor)
    }
}

#[cfg(feature = "std")]
impl<K, V> KeyValues for collections::BTreeMap<K, V>
where
    for<'a> (&'a K, &'a V): KeyValue,
{
    fn visit(&self, visitor: &mut dyn Visitor) {
        for kv in self {
            visitor.visit_kv(&kv);
        }
    }
}

#[cfg(feature = "std")]
impl<K, V> KeyValues for collections::HashMap<K, V>
where
    for<'a> (&'a K, &'a V): KeyValue,
    K: Eq + hash::Hash,
{
    fn visit(&self, visitor: &mut dyn Visitor) {
        for kv in self {
            visitor.visit_kv(&kv);
        }
    }
}

impl<'a, T: ?Sized> KeyValues for &'a T
where
    T: KeyValues
{
    fn visit(&self, visitor: &mut dyn Visitor) {
        (*self).visit(visitor)
    }
}

impl<K, V> KeyValue for (K, V)
where
    K: ToKey,
    V: ToValue,
{
    fn key(&self) -> Key {
        self.0.to_key()
    }

    fn value(&self) -> Value {
        self.1.to_value()
    }
}

impl<'a, T: ?Sized> KeyValue for &'a T
where
    T: KeyValue
{
    fn key(&self) -> Key {
        (*self).key()
    }

    fn value(&self) -> Value {
        (*self).value()
    }
}

impl<T: serde::Serialize + fmt::Display> ToValue for T {
    fn to_value(&self) -> Value {
        Value::new(self)
    }
}

impl<'a> ToValue for &'a dyn ToValue {
    fn to_value(&self) -> Value {
        (*self).to_value()
    }
}

impl<T: fmt::Display> ToKey for T {
    fn to_key(&self) -> Key {
        Key::from_display(self)
    }
}

impl<'a> ToKey for &'a dyn ToKey {
    fn to_key(&self) -> Key {
        (*self).to_key()
    }
}

/// A single property value.
/// 
/// Values borrow their underlying data and implement `serde::Serialize`.
#[derive(Clone)]
pub struct Value<'a> {
    inner: ValueInner<'a>,
}

#[derive(Clone, Copy)]
enum ValueInner<'a> {
    Display(&'a dyn fmt::Display),
    #[cfg(feature = "erased-serde")]
    Serde(&'a dyn erased_serde::Serialize),
    #[cfg(not(feature = "erased-serde"))]
    Primitive(primitive::Primitive),
}

impl<'a> ToValue for Value<'a> {
    fn to_value(&self) -> Value {
        Value { inner: self.inner }
    }
}

impl<'a> serde::Serialize for Value<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.inner {
            ValueInner::Display(v) => serializer.collect_str(&v),

            #[cfg(feature = "erased-serde")]
            ValueInner::Serde(v) => v.serialize(serializer),

            #[cfg(not(feature = "erased-serde"))]
            ValueInner::Primitive(v) => v.serialize(serializer),
        }
    }
}

impl<'a> Value<'a> {
    /// Create a new value.
    /// 
    /// The value must implement both `serde::Serialize` and `fmt::Display`.
    /// Either implementation will be used depending on whether the standard
    /// library is available, but is exposed through the same API.
    pub fn new(v: &'a (impl serde::Serialize + fmt::Display)) -> Self {
        Value {
            inner: {
                #[cfg(feature = "erased-serde")]
                {
                    ValueInner::Serde(v)
                }

                #[cfg(not(feature = "erased-serde"))]
                {
                    // Try capture a primitive value
                    // If we can represent it on the stack then we can avoid using
                    // the `Display` implementation
                    if let Some(primitive) = primitive::Primitive::try_from(v) {
                        ValueInner::Primitive(primitive)
                    } else {
                        ValueInner::Display(v)
                    }
                }
            }
        }
    }

    pub fn from_display(v: &'a impl fmt::Display) -> Self {
        Value {
            inner: ValueInner::Display(v),
        }
    }

    #[cfg(feature = "erased-serde")]
    pub fn from_serde(v: &'a impl serde::Serialize) -> Self {
        Value {
            inner: ValueInner::Serde(v),
        }
    }
}

impl<'a> fmt::Debug for Value<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Value").finish()
    }
}

/// A single property key.
/// 
/// `Key`s borrow their underlying data and implement `serde::Serialize`.
#[derive(Clone)]
pub struct Key<'a> {
    inner: KeyInner<'a>,
}

#[derive(Clone, Copy)]
enum KeyInner<'a> {
    Display(&'a dyn fmt::Display),
}

impl<'a> Key<'a> {
    pub fn from_display(key: &'a impl fmt::Display) -> Self {
        Key {
            inner: KeyInner::Display(key),
        }
    }
}

impl<'a> serde::Serialize for Key<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.inner {
            KeyInner::Display(v) => serializer.collect_str(&v),
        }
    }
}

impl<'a> fmt::Debug for Key<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.inner {
            KeyInner::Display(k) => write!(f, "{}", k)
        }
    }
}

impl<'a> fmt::Display for Key<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.inner {
            KeyInner::Display(k) => write!(f, "{}", k)
        }
    }
}

pub(crate) struct EmptyKeyValues;

impl KeyValues for EmptyKeyValues {
    fn visit(&self, visitor: &mut dyn Visitor) { }
}

#[doc(hidden)]
pub struct RawKeyValues<'a>(pub &'a [(&'a str, &'a dyn ToValue)]);

impl<'a> fmt::Debug for RawKeyValues<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RawKeyValues").finish()
    }
}

impl<'a> KeyValues for RawKeyValues<'a> {
    fn visit(&self, visitor: &mut dyn Visitor) {
        self.0.visit(visitor)
    }
}

/// Serialize key values as a map.
pub trait AsMap {
    fn as_map(&self) -> Map<&Self>;
}

impl<KVS> AsMap for KVS where KVS: KeyValues {
    fn as_map(&self) -> Map<&KVS> {
        Map::new(&self)
    }
}

/// A `serde` adapter to serialize key values as entries in a map.
/// 
/// If this type wraps a `serde` serializer then it can be used as a serializer
/// for key value pairs.
/// 
/// If this type wraps a set of key value pairs then it can be serialized itself
/// using `serde`.
pub struct Map<KVS>(KVS);

impl<KVS> Map<KVS> {
    pub fn new(kvs: KVS) -> Self {
        Map(kvs)
    }
}

impl<KVS> serde::Serialize for Map<KVS> where KVS: KeyValues {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer
    {
        use serde::ser::SerializeMap;

        struct SerdeVisitor<M>(M);
        impl<M> Visitor for SerdeVisitor<M> where M: SerializeMap {
            fn visit_kv(&mut self, kv: &dyn KeyValue) {
                let _ = SerializeMap::serialize_entry(&mut self.0, &kv.key(), &kv.value());
            }
        }

        let mut map = SerdeVisitor(serializer.serialize_map(None)?);

        KeyValues::visit(&self.0, &mut map);

        map.0.end()
    }
}

impl<KVS> fmt::Debug for Map<KVS> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Map").finish()
    }
}

/// Serialize key values as a sequence.
pub trait AsSeq {
    fn as_seq(&self) -> Seq<&Self>;
}

impl<KVS> AsSeq for KVS where KVS: KeyValues {
    fn as_seq(&self) -> Seq<&Self> {
        Seq::new(self)
    }
}

/// A `serde` adapter to serialize key values as tuple elements in a sequence.
/// 
/// If this type wraps a `serde` serializer then it can be used as a serializer
/// for key value pairs.
/// 
/// If this type wraps a set of key value pairs then it can be serialized itself
/// using `serde`.
pub struct Seq<KVS>(KVS);

impl<KVS> Seq<KVS> {
    pub fn new(kvs: KVS) -> Self {
        Seq(kvs)
    }
}

impl<KVS> serde::Serialize for Seq<KVS> where KVS: KeyValues {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer
    {
        use serde::ser::SerializeSeq;

        struct SerdeVisitor<M>(M);
        impl<S> Visitor for SerdeVisitor<S> where S: SerializeSeq {
            fn visit_kv(&mut self, kv: &dyn KeyValue) {
                let _ = SerializeSeq::serialize_element(&mut self.0, &(kv.key(), kv.value()));
            }
        }

        let mut seq = SerdeVisitor(serializer.serialize_seq(None)?);

        KeyValues::visit(&self.0, &mut seq);

        seq.0.end()
    }
}

impl<KVS> fmt::Debug for Seq<KVS> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Seq").finish()
    }
}

#[cfg(test)]
pub(crate) mod test {
    extern crate serde_test;
    use self::serde_test::{assert_de_tokens, assert_de_tokens_error, assert_tokens, Token};

    use super::*;
    
    pub(crate) struct CheckKeyValues<F>(F);

    impl<F> Serializer for CheckKeyValues<F>
    where
        F: FnMut(&str, Value),
    {
        fn visit_kv(&mut self, kv: &dyn KeyValue) {
            self.0(kv.key(), kv.value());
        }
    }
}