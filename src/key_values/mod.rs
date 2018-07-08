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
    /// 
    /// This would usually mean iterating through some collection of [`KeyValue`]s
    /// and calling `visit` on each of them.
    fn visit(&self, visitor: &mut dyn Visitor);
}

/// A single key value pair.
pub trait KeyValue {
    /// Serialize the key value pair.
    /// 
    /// This would usually mean calling [`Visitor::visit_key`] and [`Visitor::visit_value`] on some internal
    /// key and value.
    fn visit(&self, visitor: &mut dyn Visitor);
}

/// A visitor for key value pairs.
pub trait Visitor {
    /// Visit a key.
    /// 
    /// Calling `visit_key` multiple times in a row is incorrect and allowed to panic
    /// or produce bogus results.
    fn visit_key(&mut self, k: Key);

    /// Visit a value.
    /// 
    /// Calling `visit_value` before `visit_key`, or multiple times in a row is
    /// incorrect and allowed to panic or produce bogus results.
    fn visit_value(&mut self, v: Value);
}

impl<KV> KeyValues for [KV] where KV: KeyValue {
    fn visit(&self, visitor: &mut dyn Visitor) {
        for kv in self {
            kv.visit(visitor);
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
            kv.visit(visitor);
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
            kv.visit(visitor);
        }
    }
}

impl<'a, T: ?Sized> Visitor for &'a mut T
where
    T: Visitor,
{
    fn visit_key(&mut self, k: Key) {
        (*self).visit_key(k)
    }

    fn visit_value(&mut self, v: Value) {
        (*self).visit_value(v)
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
    fn visit(&self, visitor: &mut dyn Visitor) {
        visitor.visit_key(self.0.to_key());
        visitor.visit_value(self.1.to_value());
    }
}

impl<'a, T: ?Sized> KeyValue for &'a T
where
    T: KeyValue,
{
    fn visit(&self, visitor: &mut dyn Visitor) {
        (*self).visit(visitor)
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
/// 
/// If this type wraps a `serde` serializer then it can be used as a serializer
/// for key value pairs.
/// 
/// If this type wraps a set of key value pairs then it can be serialized itself
/// using `serde`.
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
        impl<M> Visitor for SerdeVisitor<M> where M: SerializeMap {
            fn visit_key(&mut self, k: Key) {
                let _ = SerializeMap::serialize_key(&mut self.0, &k);
            }

            fn visit_value(&mut self, v: Value) {
                let _ = SerializeMap::serialize_key(&mut self.0, &v);
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
