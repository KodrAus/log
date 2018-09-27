//! Structured properties for log records.
//! 
//! Structured logging in `log` is made up of a few traits:
//! 
//! - [`KeyValues`]: A set of [`KeyValue`]s
//! - [`KeyValue`]: A single [`Key`] and [`Value`] pair
//! - [`ToKey`]: Any type that can be converted into a [`Key`]
//! - [`ToValue`]: Any type that can be converted into a [`Value`]
//! - [`Visitor`]: A type that can visit [`Key`]s and [`Value`]s.
//! Visitors are driven by [`KeyValues`].
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

/// A set of key value pairs that can be serialized.
pub trait KeyValues {
    /// Serialize the key value pairs.
    /// 
    /// This would usually mean iterating through some collection of [`KeyValue`]s
    /// and calling `visit` on each of them.
    fn visit<'kvs, 'vis>(&'kvs self, visitor: &'vis mut dyn Visitor<'kvs>);
}

/// A single key value pair.
pub trait KeyValue {
    /// Serialize the key value pair.
    /// 
    /// This would usually mean calling [`Visitor::visit_pair`] on some internal
    /// key and value.
    fn visit<'kvs, 'vis>(&'kvs self, visitor: &'vis mut dyn Visitor<'kvs>);
}

/// A visitor for key value pairs.
/// 
/// The lifetime of the keys and values is captured by the `'kvs` type.
pub trait Visitor<'kvs> {
    /// Visit a key value pair.
    fn visit_pair<'vis>(&'vis mut self, k: Key<'kvs>, v: Value<'kvs>);
}

impl<KV> KeyValues for [KV] where KV: KeyValue {
    fn visit<'kvs, 'vis>(&'kvs self, visitor: &'vis mut dyn Visitor<'kvs>)
    {
        for kv in self {
            kv.visit(visitor);
        }
    }
}

#[cfg(feature = "std")]
impl<KV> KeyValues for Vec<KV> where KV: KeyValue {
    fn visit<'kvs, 'vis>(&'kvs self, visitor: &'vis mut dyn Visitor<'kvs>)
    {
        self.as_slice().visit(visitor)
    }
}

#[cfg(feature = "std")]
impl<K, V> KeyValues for collections::BTreeMap<K, V>
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
impl<K, V> KeyValues for collections::HashMap<K, V>
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

impl<'a, T: ?Sized> KeyValues for &'a T
where
    T: KeyValues,
{
    fn visit<'kvs, 'vis>(&'kvs self, visitor: &'vis mut dyn Visitor<'kvs>)
    {
        (*self).visit(visitor)
    }
}

impl<K, V> KeyValue for (K, V)
where
    K: ToKey,
    V: ToValue,
{
    fn visit<'kvs, 'vis>(&'kvs self, visitor: &'vis mut dyn Visitor<'kvs>)
    {
        visitor.visit_pair(self.0.to_key(), self.1.to_value());
    }
}

impl<'a, T: ?Sized> KeyValue for &'a T
where
    T: KeyValue,
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
pub trait AsMap {
    /// Get a `Map` that can be serialized using `serde`.
    fn as_map(&self) -> Map<&Self>;
}

impl<KVS> AsMap for KVS where KVS: KeyValues {
    fn as_map(&self) -> Map<&KVS> {
        Map::new(&self)
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

impl<KVS> serde::Serialize for Map<KVS> where KVS: KeyValues {
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

        KeyValues::visit(&self.0, &mut map);

        map.0.end()
    }
}

impl<KVS> fmt::Debug for Map<KVS> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Map").finish()
    }
}

#[doc(hidden)]
pub struct RawKeyValues<'a>(pub &'a [(&'a str, &'a dyn ToValue)]);

impl<'a> fmt::Debug for RawKeyValues<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RawKeyValues").finish()
    }
}

impl<'a> KeyValues for RawKeyValues<'a> {
    fn visit<'kvs, 'vis>(&'kvs self, visitor: &'vis mut dyn Visitor<'kvs>)
    {
        self.0.visit(visitor)
    }
}
