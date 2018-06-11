//! Log record properties.

mod macros;
mod value;

pub mod adapter;

pub use self::value::*;

use std::fmt;

use serde;

/// A serializer for key value pairs.
pub trait Serializer {
    /// Serialize the key and value.
    fn serialize_kv(&mut self, kv: &KeyValue);
}

/// A set of key value pairs that can be serialized.
pub trait KeyValues {
    /// Serialize the key value pairs.
    fn serialize(&self, serializer: &mut Serializer);
}

/// A single key value pair.
pub trait KeyValue {
    /// Get the key.
    fn key(&self) -> &str;
    /// Get the value.
    fn value(&self) -> Value;
}

impl<K, V> KeyValue for (K, V)
where
    K: AsRef<str>,
    V: ToValue,
{
    fn key(&self) -> &str {
        self.0.as_ref()
    }

    fn value(&self) -> Value {
        self.1.to_value()
    }
}

impl<'a, T: ?Sized> KeyValue for &'a T
where
    T: KeyValue
{
    fn key(&self) -> &str {
        (*self).key()
    }

    fn value(&self) -> Value {
        (*self).value()
    }
}

impl<'a, T: ?Sized, KV> KeyValues for &'a T
where
    &'a T: IntoIterator<Item = KV>,
    KV: KeyValue
{
    fn serialize(&self, serializer: &mut Serializer) {
        for kv in self.into_iter() {
            serializer.serialize_kv(&kv);
        }
    }
}

#[derive(Debug)]
pub struct SerializeMap<T>(T);

impl<T> SerializeMap<T> {
    pub fn new(inner: T) -> Self {
        SerializeMap(inner)
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Serializer for SerializeMap<T>
    where
        T: serde::ser::SerializeMap
{
    fn serialize_kv(&mut self, kv: &KeyValue) {
        let _ = serde::ser::SerializeMap::serialize_entry(&mut self.0, kv.key(), &kv.value());
    }
}

impl<KV> serde::Serialize for SerializeMap<KV>
    where
        KV: KeyValues,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer
    {
        use serde::ser::SerializeMap as SerializeTrait;

        let mut map = SerializeMap::new(serializer.serialize_map(None)?);

        KeyValues::serialize(&self.0, &mut map);

        map.into_inner().end()
    }
}

#[derive(Debug)]
pub struct SerializeSeq<T>(T);

impl<T> SerializeSeq<T> {
    pub fn new(inner: T) -> Self {
        SerializeSeq(inner)
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Serializer for SerializeSeq<T>
    where
        T: serde::ser::SerializeSeq
{
    fn serialize_kv(&mut self, kv: &KeyValue) {
        let _ = serde::ser::SerializeSeq::serialize_element(&mut self.0, &(kv.key(), kv.value()));
    }
}

impl<KV> serde::Serialize for SerializeSeq<KV>
    where
        KV: KeyValues,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer
    {
        use serde::ser::SerializeSeq as SerializeTrait;

        let mut seq = SerializeSeq::new(serializer.serialize_seq(None)?);

        KeyValues::serialize(&self.0, &mut seq);

        seq.into_inner().end()
    }
}

#[doc(hidden)]
pub struct RawKeyValues<'a>(pub &'a [(&'a str, Value<'a>)]);

impl<'a> fmt::Debug for RawKeyValues<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RawKeyValues").finish()
    }
}

impl<'a> KeyValues for RawKeyValues<'a> {
    fn serialize(&self, serializer: &mut Serializer) {
        self.0.serialize(serializer)
    }
}

/// A chain of properties.
#[derive(Clone)]
pub struct Properties<'a> {
    kvs: &'a KeyValues,
    parent: Option<&'a Properties<'a>>,
}

impl<'a> Properties<'a> {
    pub(crate) fn root(properties: &'a KeyValues) -> Self {
        Properties {
            kvs: properties,
            parent: None
        }
    }

    pub(crate) fn chained(properties: &'a KeyValues, parent: &'a Properties) -> Self {
        Properties {
            kvs: properties,
            parent: Some(parent)
        }
    }

    pub fn serialize_map(&self) -> SerializeMap<&Self> {
        SerializeMap::new(&self)
    }

    pub fn serialize_seq(&self) -> SerializeSeq<&Self> {
        SerializeSeq::new(&self)
    }
}

impl<'a> KeyValues for Properties<'a> {
    fn serialize(&self, serializer: &mut Serializer) {
        self.kvs.serialize(serializer);

        if let Some(parent) = self.parent {
            parent.serialize(serializer);
        }
    }
}

impl<'a> fmt::Debug for Properties<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Properties").finish()
    }
}

impl<'a> Default for Properties<'a> {
    fn default() -> Self {
        Properties {
            kvs: &RawKeyValues(&[]),
            parent: None,
        }
    }
}