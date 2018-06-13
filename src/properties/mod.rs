//! Log record properties.

#[macro_use]
mod macros;

pub mod adapter;

#[cfg(feature = "std")]
use std::collections;
#[cfg(feature = "std")]
use std::hash;

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

impl<KV> KeyValues for [KV] where KV: KeyValue {
    fn serialize(&self, serializer: &mut dyn Serializer) {
        for kv in self {
            serializer.serialize_kv(&kv);
        }
    }
}

#[cfg(feature = "std")]
impl<KV> KeyValues for Vec<KV> where KV: KeyValue {
    fn serialize(&self, serializer: &mut dyn Serializer) {
        self.as_slice().serialize(serializer)
    }
}

#[cfg(feature = "std")]
impl<K, V> KeyValues for collections::BTreeMap<K, V>
where
    for<'a> (&'a K, &'a V): KeyValue,
{
    fn serialize(&self, serializer: &mut dyn Serializer) {
        for kv in self {
            serializer.serialize_kv(&kv);
        }
    }
}

#[cfg(feature = "std")]
impl<K, V> KeyValues for collections::HashMap<K, V>
where
    for<'a> (&'a K, &'a V): KeyValue,
    K: Eq + hash::Hash,
{
    fn serialize(&self, serializer: &mut dyn Serializer) {
        for kv in self {
            serializer.serialize_kv(&kv);
        }
    }
}

impl<'a, T: ?Sized> KeyValues for &'a T
where
    T: KeyValues
{
    fn serialize(&self, serializer: &mut dyn Serializer) {
        (*self).serialize(serializer)
    }
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

/// A chain of key value pairs.
#[derive(Clone)]
pub(crate) struct Chained<'a> {
    kvs: &'a dyn KeyValues,
    parent: Option<&'a Chained<'a>>,
}

impl<'a> Chained<'a> {
    /// Create a new set of properties with no key value pairs.
    pub fn empty() -> Self {
        Chained {
            kvs: &EmptyKeyValues,
            parent: None,
        }
    }

    /// Create a new set of properties with the given initial key value pairs.
    pub fn root(properties: &'a dyn KeyValues) -> Self {
        Chained {
            kvs: properties,
            parent: None
        }
    }

    /// Create a new set of properties with a parent and additional key value pairs.
    pub fn chained(properties: &'a dyn KeyValues, parent: &'a Chained) -> Self {
        Chained {
            kvs: properties,
            parent: Some(parent)
        }
    }
}

impl<'a> KeyValues for Chained<'a> {
    fn serialize(&self, serializer: &mut dyn Serializer) {
        self.kvs.serialize(serializer);

        if let Some(parent) = self.parent {
            parent.serialize(serializer);
        }
    }
}

impl<'a> fmt::Debug for Chained<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Chained").finish()
    }
}

impl<'a> Default for Chained<'a> {
    fn default() -> Self {
        Chained::empty()
    }
}

/// A set of key value pairs associated with a log record.
pub struct Properties<'a> {
    kvs: &'a dyn KeyValues,
}

impl<'a> Properties<'a> {
    pub(crate) fn new(kvs: &'a dyn KeyValues) -> Self {
        Properties {
            kvs,
        }
    }

    /// Get the raw key values.
    pub fn key_values(&self) -> &dyn KeyValues {
        &self.kvs
    }

    /// Get a wrapper over these properties that can be serialized using `serde`.
    /// 
    /// The properties will be serialized as a flat map of key value entries.
    pub fn as_map(&self) -> Map {
        Map(&self.kvs)
    }

    /// Get a wrapper over these properties that can be serialized using `serde`.
    /// 
    /// The properties will be serialized as a flat sequence of key value tuples.
    pub fn as_seq(&self) -> Seq {
        Seq(&self.kvs)
    }

    /// Count the number of key value pairs.
    pub fn count(&self) -> usize {
        struct Counter(usize);

        impl Serializer for Counter {
            fn serialize_kv(&mut self, kv: &dyn KeyValue) {
                self.0 += 1;
            }
        }

        let mut counter = Counter(0);
        self.kvs.serialize(&mut counter);

        counter.0
    }
}

impl<'a> fmt::Debug for Properties<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Properties").finish()
    }
}

/// A `serde` adapter to serialize key values as entries in a map.
/// 
/// If this type wraps a `serde` serializer then it can be used as a serializer
/// for key value pairs.
/// 
/// If this type wraps a set of key value pairs then it can be serialized itself
/// using `serde`.
pub struct Map<'a>(&'a dyn KeyValues);

impl<'a> serde::Serialize for Map<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer
    {
        use serde::ser::SerializeMap;

        struct SerdeSerializer<M>(M);
        impl<M> Serializer for SerdeSerializer<M> where M: SerializeMap {
            fn serialize_kv(&mut self, kv: &dyn KeyValue) {
                let _ = SerializeMap::serialize_entry(&mut self.0, kv.key(), &kv.value());
            }
        }

        let mut map = SerdeSerializer(serializer.serialize_map(None)?);

        KeyValues::serialize(&self.0, &mut map);

        map.0.end()
    }
}

impl<'a> fmt::Debug for Map<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Map").finish()
    }
}

/// A `serde` adapter to serialize key values as tuple elements in a sequence.
/// 
/// If this type wraps a `serde` serializer then it can be used as a serializer
/// for key value pairs.
/// 
/// If this type wraps a set of key value pairs then it can be serialized itself
/// using `serde`.
pub struct Seq<'a>(&'a dyn KeyValues);

impl<'a> serde::Serialize for Seq<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer
    {
        use serde::ser::SerializeSeq;

        struct SerdeSerializer<M>(M);
        impl<S> Serializer for SerdeSerializer<S> where S: SerializeSeq {
            fn serialize_kv(&mut self, kv: &dyn KeyValue) {
                let _ = SerializeSeq::serialize_element(&mut self.0, &(kv.key(), kv.value()));
            }
        }

        let mut seq = SerdeSerializer(serializer.serialize_seq(None)?);

        KeyValues::serialize(&self.0, &mut seq);

        seq.0.end()
    }
}

impl<'a> fmt::Debug for Seq<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Seq").finish()
    }
}
