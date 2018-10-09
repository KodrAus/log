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
mod primitive;
mod error;

pub mod adapter;

#[cfg(feature = "std")]
use std::collections;
#[cfg(feature = "std")]
use std::hash;

use serde::{Serialize, Serializer};

pub use self::error::Error;
pub use self::key::{Key, ToKey};
pub use self::value::{Value, ToValue};

/// A visitor for key value pairs.
/// 
/// The lifetime of the keys and values is captured by the `'kvs` type.
pub trait Visitor<'kvs> {
    /// Visit a key value pair.
    fn visit_pair(&mut self, k: Key<'kvs>, v: Value<'kvs>) -> Result<(), Error>;
}

impl<'a, 'kvs, T: ?Sized> Visitor<'kvs> for &'a mut T
where
    T: Visitor<'kvs>,
{
    fn visit_pair(&mut self, k: Key<'kvs>, v: Value<'kvs>) -> Result<(), Error> {
        (*self).visit_pair(k, v)
    }
}

/// A source for key value pairs that can be serialized.
pub trait KeyValueSource {
    /// Serialize the key value pairs.
    fn visit<'kvs>(&'kvs self, visitor: &mut Visitor<'kvs>) -> Result<(), Error>;

    /// An adapter to erase self.
    fn erase(&self) -> &dyn KeyValueSource
    where
        Self: Sized,
    {
        self
    }
}

impl<'a> dyn KeyValueSource + 'a {
    /// An adapter to borrow self.
    pub fn as_ref(&self) -> &Self {
        self
    }

    /// Chain two `KeyValueSource`s together.
    pub fn chain<KVS>(&self, other: KVS) -> Chained<&Self, KVS> {
        Chained(self, other)
    }

    /// Apply a function to each key-value pair.
    pub fn try_for_each<F, E>(&self, f: F) -> Result<(), Error>
    where
        F: FnMut(Key, Value) -> Result<(), E>,
        E: Into<Error>,
    {
        struct ForEach<F, E>(F, std::marker::PhantomData<E>);

        impl<'kvs, F, E> Visitor<'kvs> for ForEach<F, E>
        where
            F: FnMut(Key, Value) -> Result<(), E>,
            E: Into<Error>,
        {
            fn visit_pair(&mut self, k: Key<'kvs>, v: Value<'kvs>) -> Result<(), Error> {
                (self.0)(k, v).map_err(Into::into)
            }
        }

        self.visit(&mut ForEach(f, Default::default()))
    }

    /// Serialize the key-value pairs as a map.
    pub fn serialize_as_map(&self) -> SerializeAsMap<&Self> {
        SerializeAsMap(self)
    }

    /// Sort the inner key value pairs, retaining the last for each key.
    /// 
    /// This method requires allocating a map to sort the keys.
    #[cfg(feature = "std")]
    fn sort_retain_last(&self) -> SortRetainLast<&Self> {
        SortRetainLast(self)
    }
}

/// A chain of two `KeyValueSource`s.
pub struct Chained<A, B>(A, B);

impl<A, B> KeyValueSource for Chained<A, B>
where
    A: KeyValueSource,
    B: KeyValueSource,
{
    fn visit<'kvs>(&'kvs self, visitor: &mut Visitor<'kvs>) -> Result<(), Error> {
        self.0.visit(visitor)?;
        self.1.visit(visitor)?;

        Ok(())
    }
}

/// Sort the inner key values, retaining the last one with a given key.
#[derive(Debug)]
#[cfg(feature = "std")]
pub struct SortRetainLast<KVS>(KVS);

impl<KVS> KeyValueSource for SortRetainLast<KVS>
where
    KVS: KeyValueSource,
{
    fn visit<'kvs>(&'kvs self, visitor: &mut Visitor<'kvs>) -> Result<(), Error> {
        use std::collections::BTreeMap;

        struct Seen<'kvs>(BTreeMap<Key<'kvs>, Value<'kvs>>);

        impl<'kvs> Visitor<'kvs> for Seen<'kvs> {
            fn visit_pair(&mut self, k: Key<'kvs>, v: Value<'kvs>) -> Result<(), Error> {
                self.0.insert(k, v);

                Ok(())
            }
        }

        let mut seen = Seen(BTreeMap::new());
        self.0.visit(&mut seen)?;

        for (k, v) in seen.0 {
            visitor.visit_pair(k, v)?;
        }

        Ok(())
    }
}

/// Serialize the key-value pairs as a map.
#[derive(Debug)]
pub struct SerializeAsMap<KVS>(KVS);

impl<'a, KVS> Serialize for SerializeAsMap<KVS>
where
    KVS: KeyValueSource + 'a,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(None)?;

        self.0
           .erase()
           .try_for_each(|k, v| map.serialize_entry(&k, &v))
           .map_err(Error::into_serde)?;

        map.end()
    }
}

impl<K, V> KeyValueSource for (K, V)
where
    K: ToKey,
    V: ToValue,
{
    fn visit<'kvs>(&'kvs self, visitor: &mut Visitor<'kvs>) -> Result<(), Error> {
        visitor.visit_pair(self.0.to_key(), self.1.to_value())
    }
}

impl<KV> KeyValueSource for [KV] where KV: KeyValueSource {
    fn visit<'kvs>(&'kvs self, visitor: &mut Visitor<'kvs>) -> Result<(), Error> {
        for kv in self {
            kv.visit(visitor)?;
        }

        Ok(())
    }
}

#[cfg(feature = "std")]
impl<KV> KeyValueSource for Vec<KV> where KV: KeyValueSource {
    fn visit<'kvs>(&'kvs self, visitor: &mut Visitor<'kvs>) -> Result<(), Error> {
        self.as_slice().visit(visitor)
    }
}

#[cfg(feature = "std")]
impl<K, V> KeyValueSource for collections::BTreeMap<K, V>
where
    K: ToKey,
    V: ToValue,
{
    fn visit<'kvs>(&'kvs self, visitor: &mut Visitor<'kvs>) -> Result<(), Error> {
        for (k, v) in self {
            visitor.visit_pair(k.to_key(), v.to_value())?;
        }

        Ok(())
    }
}

#[cfg(feature = "std")]
impl<K, V> KeyValueSource for collections::HashMap<K, V>
where
    K: ToKey + Eq + hash::Hash,
    V: ToValue,
{
    fn visit<'kvs>(&'kvs self, visitor: &mut Visitor<'kvs>) -> Result<(), Error> {
        for (k, v) in self {
            visitor.visit_pair(k.to_key(), v.to_value())?;
        }

        Ok(())
    }
}

impl<'a, T: ?Sized> KeyValueSource for &'a T
where
    T: KeyValueSource,
{
    fn visit<'kvs>(&'kvs self, visitor: &mut Visitor<'kvs>) -> Result<(), Error> {
        (*self).visit(visitor)
    }
}
