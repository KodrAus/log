//! Log record properties.

#[macro_use]
mod macros;
mod primitive;
mod key;
mod value;

pub mod adapter;

#[cfg(feature = "std")]
use std::collections;
#[cfg(feature = "std")]
use std::hash;
use std::fmt;

use serde;

pub use self::key::{Key, ToKey};
pub use self::value::{Value, ToValue};

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

/// A visitor for key value pairs.
pub trait Visitor {
    /// Visit the key and value.
    fn visit_kv(&mut self, kv: &dyn KeyValue);
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
    T: KeyValues,
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
    T: KeyValue,
{
    fn key(&self) -> Key {
        (*self).key()
    }

    fn value(&self) -> Value {
        (*self).value()
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
            S: serde::Serializer,
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
            S: serde::Serializer,
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
