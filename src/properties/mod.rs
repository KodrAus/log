//! Log record properties.

#[macro_use]
mod macros;

pub mod adapter;

use std::fmt;

use serde;
#[cfg(feature = "erased-serde")]
use erased_serde;

/// A serializer for key value pairs.
pub trait Serializer {
    /// Serialize the key and value.
    fn serialize_kv(&mut self, kv: &dyn KeyValue);
}

/// A set of key value pairs that can be serialized.
pub trait KeyValues {
    /// Serialize the key value pairs.
    fn serialize(&self, serializer: &mut dyn Serializer);
}

/// A single key value pair.
pub trait KeyValue {
    /// Get the key.
    fn key(&self) -> &str;
    /// Get the value.
    fn value(&self) -> Value;
}

/// Converting into a `Value`.
pub trait ToValue {
    /// Perform the conversion.
    fn to_value(&self) -> Value;
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
    fn serialize(&self, serializer: &mut dyn Serializer) {
        for kv in self.into_iter() {
            serializer.serialize_kv(&kv);
        }
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

/// A single property value.
/// 
/// Values implement `serde::Serialize`.
pub struct Value<'a> {
    inner: ValueInner<'a>,
}

#[derive(Clone, Copy)]
enum ValueInner<'a> {
    Fmt(&'a dyn fmt::Display),
    #[cfg(feature = "erased-serde")]
    Serde(&'a dyn erased_serde::Serialize),
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
            ValueInner::Fmt(v) => serializer.collect_str(&v),
            #[cfg(feature = "erased-serde")]
            ValueInner::Serde(v) => v.serialize(serializer),
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
                    ValueInner::Fmt(v)
                }
            }
        }
    }

    pub fn fmt(v: &'a impl fmt::Display) -> Self {
        Value {
            inner: ValueInner::Fmt(v),
        }
    }

    #[cfg(feature = "erased-serde")]
    pub fn serde(v: &'a impl serde::Serialize) -> Self {
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

/// A `serde` adapter to serialize key values as entries in a map.
/// 
/// If this type wraps a `serde` serializer then it can be used as a serializer
/// for key value pairs.
/// 
/// If this type wraps a set of key value pairs then it can be serialized itself
/// using `serde`.
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
    fn serialize_kv(&mut self, kv: &dyn KeyValue) {
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

/// A `serde` adapter to serialize key values as tuple elements in a sequence.
/// 
/// If this type wraps a `serde` serializer then it can be used as a serializer
/// for key value pairs.
/// 
/// If this type wraps a set of key value pairs then it can be serialized itself
/// using `serde`.
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
    fn serialize_kv(&mut self, kv: &dyn KeyValue) {
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

pub(crate) struct EmptyKeyValues;

impl KeyValues for EmptyKeyValues {
    fn serialize(&self, serializer: &mut dyn Serializer) { }
}

#[doc(hidden)]
pub struct RawKeyValues<'a>(pub &'a [(&'a str, &'a dyn ToValue)]);

impl<'a> fmt::Debug for RawKeyValues<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RawKeyValues").finish()
    }
}

impl<'a> KeyValues for RawKeyValues<'a> {
    fn serialize(&self, serializer: &mut dyn Serializer) {
        self.0.serialize(serializer)
    }
}

/// A chain of properties.
#[derive(Clone)]
pub struct Properties<'a> {
    kvs: &'a dyn KeyValues,
    parent: Option<&'a Properties<'a>>,
}

impl<'a> Properties<'a> {
    /// Create a new set of properties with no key value pairs.
    pub(crate) fn empty() -> Self {
        Properties {
            kvs: &EmptyKeyValues,
            parent: None,
        }
    }

    /// Create a new set of properties with the given initial key value pairs.
    pub(crate) fn root(properties: &'a dyn KeyValues) -> Self {
        Properties {
            kvs: properties,
            parent: None
        }
    }

    /// Create a new set of properties with a parent and additional key value pairs.
    pub(crate) fn chained(properties: &'a dyn KeyValues, parent: &'a Properties) -> Self {
        Properties {
            kvs: properties,
            parent: Some(parent)
        }
    }

    /// Get a wrapper over these properties that can be serialized using `serde`.
    /// 
    /// The properties will be serialized as a flat map of key value entries.
    pub fn serialize_map(&self) -> SerializeMap<&Self> {
        SerializeMap::new(&self)
    }

    /// Get a wrapper over these properties that can be serialized using `serde`.
    /// 
    /// The properties will be serialized as a flat sequence of key value tuples.
    pub fn serialize_seq(&self) -> SerializeSeq<&Self> {
        SerializeSeq::new(&self)
    }
}

impl<'a> KeyValues for Properties<'a> {
    fn serialize(&self, serializer: &mut dyn Serializer) {
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
        Properties::empty()
    }
}