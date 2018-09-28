//! Structured properties for log records.
//! 
//! Structured logging in `log` is made up of a few traits:
//! 
//! - [`KeyValueSource`]: A set of [`KeyValue`]s
//! - [`KeyValue`]: A single [`Key`] and [`Value`] pair
//! - [`ToKey`]: Any type that can be converted into a [`Key`]
//! - [`ToValue`]: Any type that can be converted into a [`Value`]
//! - [`Visitor`]: A type that can visit [`Key`]s and [`Value`]s.
//! Visitors are driven by [`KeyValueSource`].
//! 
//! Structured logging uses `serde` for serializing key value pairs.

#[macro_use]
mod macros;
mod key;
mod value;
#[cfg(not(feature = "erased-serde"))]
mod primitive;
#[cfg(feature = "std")]
mod misc;

pub mod adapter;

#[cfg(feature = "std")]
use std::collections;
#[cfg(feature = "std")]
use std::hash;
use std::fmt;

use serde;

pub use self::key::{Key, ToKey};
pub use self::value::{Value, ToValue};

/// A source for key value pairs that can be serialized.
pub trait KeyValueSource {
    /// Serialize the key value pairs.
    fn visit<'kvs, 'vis>(&'kvs self, visitor: &'vis mut dyn Visitor<'kvs>);
}

/// A visitor for key value pairs.
/// 
/// The lifetime of the keys and values is captured by the `'kvs` type.
pub trait Visitor<'kvs> {
    /// Visit a key value pair.
    fn visit_pair<'vis>(&'vis mut self, k: Key<'kvs>, v: Value<'kvs>);
}

impl<K, V> KeyValueSource for (K, V)
where
    K: ToKey,
    V: ToValue,
{
    fn visit<'kvs, 'vis>(&'kvs self, visitor: &'vis mut dyn Visitor<'kvs>)
    {
        visitor.visit_pair(self.0.to_key(), self.1.to_value());
    }
}

impl<KV> KeyValueSource for [KV] where KV: KeyValueSource {
    fn visit<'kvs, 'vis>(&'kvs self, visitor: &'vis mut dyn Visitor<'kvs>)
    {
        for kv in self {
            kv.visit(visitor);
        }
    }
}

#[cfg(feature = "std")]
impl<KV> KeyValueSource for Vec<KV> where KV: KeyValueSource {
    fn visit<'kvs, 'vis>(&'kvs self, visitor: &'vis mut dyn Visitor<'kvs>)
    {
        self.as_slice().visit(visitor)
    }
}

#[cfg(feature = "std")]
impl<K, V> KeyValueSource for collections::BTreeMap<K, V>
where
    K: ToKey,
    V: ToValue,
{
    fn visit<'kvs, 'vis>(&'kvs self, visitor: &'vis mut dyn Visitor<'kvs>)
    {
        for (k, v) in self {
            visitor.visit_pair(k.to_key(), v.to_value());
        }
    }
}

#[cfg(feature = "std")]
impl<K, V> KeyValueSource for collections::HashMap<K, V>
where
    K: ToKey + Eq + hash::Hash,
    V: ToValue,
{
    fn visit<'kvs, 'vis>(&'kvs self, visitor: &'vis mut dyn Visitor<'kvs>)
    {
        for (k, v) in self {
            visitor.visit_pair(k.to_key(), v.to_value());
        }
    }
}

impl<'a, T: ?Sized> KeyValueSource for &'a T
where
    T: KeyValueSource,
{
    fn visit<'kvs, 'vis>(&'kvs self, visitor: &'vis mut dyn Visitor<'kvs>)
    {
        (*self).visit(visitor)
    }
}

impl<'a, 'kvs, T: ?Sized> Visitor<'kvs> for &'a mut T
where
    T: Visitor<'kvs>,
{
    fn visit_pair<'vis>(&'vis mut self, k: Key<'kvs>, v: Value<'kvs>) {
        (*self).visit_pair(k, v)
    }
}

/// Serialize key values as a map.
pub trait IntoMap {
    /// Get a `Map` that can be serialized using `serde`.
    fn into_map(self) -> Map<Self> where Self: Sized;
}

impl<KVS> IntoMap for KVS where KVS: KeyValueSource {
    fn into_map(self) -> Map<KVS> {
        Map::new(self)
    }
}

/// A `serde` adapter to serialize key values as entries in a map.
pub struct Map<KVS>(KVS);

impl<KVS> Map<KVS> {
    /// Create a new `Map`.
    pub fn new(kvs: KVS) -> Self {
        Map(kvs)
    }
}

impl<KVS> serde::Serialize for Map<KVS> where KVS: KeyValueSource {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        struct SerdeVisitor<M>(M);
        impl<'a, M> Visitor<'a> for SerdeVisitor<M> where M: SerializeMap {
            fn visit_pair(&mut self, k: Key<'a>, v: Value<'a>) {
                let _ = SerializeMap::serialize_entry(&mut self.0, &k, &v);
            }
        }

        let mut map = SerdeVisitor(serializer.serialize_map(None)?);

        KeyValueSource::visit(&self.0, &mut map);

        map.0.end()
    }
}

impl<KVS> fmt::Debug for Map<KVS> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Map").finish()
    }
}

#[doc(hidden)]
pub struct RawKeyValueSource<'a>(pub &'a [(&'a str, &'a dyn ToValue)]);

impl<'a> fmt::Debug for RawKeyValueSource<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RawKeyValueSource").finish()
    }
}

impl<'a> KeyValueSource for RawKeyValueSource<'a> {
    fn visit<'kvs, 'vis>(&'kvs self, visitor: &'vis mut dyn Visitor<'kvs>)
    {
        self.0.visit(visitor)
    }
}
