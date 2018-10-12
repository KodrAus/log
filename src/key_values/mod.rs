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

use std::fmt;

#[cfg(feature = "std")]
use std::collections;
#[cfg(feature = "std")]
use std::hash;
#[cfg(feature = "std")]
use std::borrow;

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
    fn visit<'kvs>(&'kvs self, visitor: &mut dyn Visitor<'kvs>) -> Result<(), Error>;

    /// Erase this `KeyValueSource` so it can be used without
    /// requiring generic type parameters.
    fn erase(&self) -> ErasedKeyValueSource
    where
        Self: Sized,
    {
        ErasedKeyValueSource::erased(self)
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
    fn get<'kvs, Q>(&'kvs self, key: Q) -> Option<Value<'kvs>>
    where
        Q: ToKey,
    {
        struct Get<'k, 'v>(Key<'k>, Option<Value<'v>>);

        impl<'k, 'kvs> Visitor<'kvs> for Get<'k, 'kvs> {
            fn visit_pair(&mut self, k: Key<'kvs>, v: Value<'kvs>) -> Result<(), Error> {
                if k == self.0 {
                    self.1 = Some(v);
                }

                Ok(())
            }
        }

        let mut visitor = Get(key.to_key(), None);
        let _ = self.visit(&mut visitor);

        visitor.1
    }

    /// An adapter to borrow self.
    fn by_ref(&self) -> &Self {
        self
    }

    /// Chain two `KeyValueSource`s together.
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
    fn serialize_as_map(self) -> SerializeAsMap<Self>
    where
        Self: Sized,
    {
        SerializeAsMap(self)
    }

    /// Sort the inner key value pairs, retaining the last for each key.
    /// 
    /// This method requires allocating a map to sort the keys.
    #[cfg(feature = "std")]
    fn sort_retain_last(self) -> SortRetainLast<Self>
    where
        Self: Sized,
    {
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
    fn visit<'kvs>(&'kvs self, visitor: &mut dyn Visitor<'kvs>) -> Result<(), Error> {
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
    fn visit<'kvs>(&'kvs self, visitor: &mut dyn Visitor<'kvs>) -> Result<(), Error> {
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

impl<KVS> Serialize for SerializeAsMap<KVS>
where
    KVS: KeyValueSource,
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

impl<K, V> KeyValueSource for (K, V)
where
    K: ToKey,
    V: ToValue,
{
    fn visit<'kvs>(&'kvs self, visitor: &mut dyn Visitor<'kvs>) -> Result<(), Error>
    {
        visitor.visit_pair(self.0.to_key(), self.1.to_value())
    }
}

impl<KVS> KeyValueSource for [KVS] where KVS: KeyValueSource {
    fn visit<'kvs>(&'kvs self, visitor: &mut dyn Visitor<'kvs>) -> Result<(), Error> {
        for kv in self {
            kv.visit(visitor)?;
        }

        Ok(())
    }
}

#[cfg(feature = "std")]
impl<KVS> KeyValueSource for Vec<KVS> where KVS: KeyValueSource {
    fn visit<'kvs>(&'kvs self, visitor: &mut dyn Visitor<'kvs>) -> Result<(), Error> {
        self.as_slice().visit(visitor)
    }
}

#[cfg(feature = "std")]
impl<K, V> KeyValueSource for collections::BTreeMap<K, V>
where
    K: borrow::Borrow<str> + Ord,
    V: ToValue,
{
    fn visit<'kvs>(&'kvs self, visitor: &mut dyn Visitor<'kvs>) -> Result<(), Error>
    {
        for (k, v) in self {
            visitor.visit_pair(k.borrow().to_key(), v.to_value())?;
        }

        Ok(())
    }

    fn get<'kvs, Q>(&'kvs self, key: Q) -> Option<Value<'kvs>>
    where
        Q: ToKey,
    {
        let key = key.to_key();
        collections::BTreeMap::get(self, key.as_ref()).map(|v| v.to_value())
    }
}

#[cfg(feature = "std")]
impl<K, V> KeyValueSource for collections::HashMap<K, V>
where
    K: borrow::Borrow<str> + Eq + hash::Hash,
    V: ToValue,
{
    fn visit<'kvs>(&'kvs self, visitor: &mut dyn Visitor<'kvs>) -> Result<(), Error>
    {
        for (k, v) in self {
            visitor.visit_pair(k.borrow().to_key(), v.to_value())?;
        }

        Ok(())
    }

    fn get<'kvs, Q>(&'kvs self, key: Q) -> Option<Value<'kvs>>
    where
        Q: ToKey,
    {
        let key = key.to_key();
        collections::HashMap::get(self, key.as_ref()).map(|v| v.to_value())
    }
}

impl<'a, T: ?Sized> KeyValueSource for &'a T
where
    T: KeyValueSource,
{
    fn visit<'kvs>(&'kvs self, visitor: &mut dyn Visitor<'kvs>) -> Result<(), Error> {
        (*self).visit(visitor)
    }

    fn get<'kvs, Q>(&'kvs self, key: Q) -> Option<Value<'kvs>>
    where
        Q: ToKey,
    {
        (*self).get(key)
    }
}

/// A key value source on a `Record`.
#[derive(Clone, Copy)]
pub struct ErasedKeyValueSource<'a>(&'a dyn ErasedKeyValueSourceBridge);

impl<'a> ErasedKeyValueSource<'a> {
    pub fn erased(kvs: &'a impl KeyValueSource) -> Self {
        ErasedKeyValueSource(kvs)
    }

    pub fn empty() -> Self {
        ErasedKeyValueSource(&(&[] as &[(&str, &dyn ToValue)]))
    }
}

impl<'a> fmt::Debug for ErasedKeyValueSource<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("KeyValueSource").finish()
    }
}

impl<'a> KeyValueSource for ErasedKeyValueSource<'a> {
    fn visit<'kvs>(&'kvs self, visitor: &mut Visitor<'kvs>) -> Result<(), Error> {
        self.0.erased_visit(visitor)
    }

    fn get<'kvs, Q>(&'kvs self, key: Q) -> Option<Value<'kvs>>
    where
        Q: ToKey,
    {
        let key = key.to_key();
        self.0.erased_get(key.as_ref())
    }
}

/// A trait that erases a `KeyValueSource` so it can be stored
/// in a `Record` without requiring any generic parameters.
trait ErasedKeyValueSourceBridge {
    fn erased_visit<'kvs>(&'kvs self, visitor: &mut dyn Visitor<'kvs>) -> Result<(), Error>;
    fn erased_get<'kvs>(&'kvs self, key: &str) -> Option<Value<'kvs>>;
}

impl<KVS> ErasedKeyValueSourceBridge for KVS
where
    KVS: KeyValueSource + ?Sized,
{
    fn erased_visit<'kvs>(&'kvs self, visitor: &mut dyn Visitor<'kvs>) -> Result<(), Error> {
        self.visit(visitor)
    }

    fn erased_get<'kvs>(&'kvs self, key: &str) -> Option<Value<'kvs>> {
        self.get(key)
    }
}