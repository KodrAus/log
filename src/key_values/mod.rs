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
use std::fmt;

pub use self::error::Error;
pub use self::key::{Key, ToKey};
pub use self::value::{Value, ToValue};

/// A visitor for key value pairs.
/// 
/// The lifetime of the keys and values is captured by the `'kvs` type.
pub trait Visitor<'kvs> {
    /// Visit a key value pair.
    fn visit_pair<'vis>(&'vis mut self, k: Key<'kvs>, v: Value<'kvs>) -> Result<(), Error>;
}

impl<'a, 'kvs, T: ?Sized> Visitor<'kvs> for &'a mut T
where
    T: Visitor<'kvs>,
{
    fn visit_pair<'vis>(&'vis mut self, k: Key<'kvs>, v: Value<'kvs>) -> Result<(), Error> {
        (*self).visit_pair(k, v)
    }
}

/// A source for key value pairs that can be serialized.
pub trait KeyValueSource {
    /// Serialize the key value pairs.
    fn visit<'kvs, V>(&'kvs self, visitor: V) -> Result<(), Error>
    where
        V: Visitor<'kvs>;

    /// An adapter to borrow self.
    fn as_ref(&self) -> &Self {
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
            fn visit_pair<'vis>(&'vis mut self, k: Key<'kvs>, v: Value<'kvs>) -> Result<(), Error> {
                (self.0)(k, v).map_err(Into::into)
            }
        }

        self.visit(ForEach(f, Default::default()))
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
    fn visit<'kvs, V>(&'kvs self, mut visitor: V) -> Result<(), Error>
    where
        V: Visitor<'kvs>
    {
        self.0.visit(&mut visitor)?;
        self.1.visit(&mut visitor)?;

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
    fn visit<'kvs, V>(&'kvs self, mut visitor: V) -> Result<(), Error>
    where
        V: Visitor<'kvs>
    {
        use std::collections::BTreeMap;

        struct Seen<'kvs>(BTreeMap<Key<'kvs>, Value<'kvs>>);

        impl<'kvs> Visitor<'kvs> for Seen<'kvs> {
            fn visit_pair<'vis>(&'vis mut self, k: Key<'kvs>, v: Value<'kvs>) -> Result<(), Error> {
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

impl<K, V> KeyValueSource for (K, V)
where
    K: ToKey,
    V: ToValue,
{
    fn visit<'kvs, VI>(&'kvs self, mut visitor: VI) -> Result<(), Error>
    where
        VI: Visitor<'kvs>
    {
        visitor.visit_pair(self.0.to_key(), self.1.to_value())
    }
}

impl<KV> KeyValueSource for [KV] where KV: KeyValueSource {
    fn visit<'kvs, V>(&'kvs self, mut visitor: V) -> Result<(), Error>
    where
        V: Visitor<'kvs>
    {
        for kv in self {
            kv.visit(&mut visitor)?;
        }

        Ok(())
    }
}

#[cfg(feature = "std")]
impl<KV> KeyValueSource for Vec<KV> where KV: KeyValueSource {
    fn visit<'kvs, V>(&'kvs self, visitor: V) -> Result<(), Error>
    where
        V: Visitor<'kvs>
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
    fn visit<'kvs, VI>(&'kvs self, mut visitor: VI) -> Result<(), Error>
    where
        VI: Visitor<'kvs>
    {
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
    fn visit<'kvs, VI>(&'kvs self, mut visitor: VI) -> Result<(), Error>
    where
        VI: Visitor<'kvs>
    {
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
    fn visit<'kvs, V>(&'kvs self, visitor: V) -> Result<(), Error>
    where
        V: Visitor<'kvs>
    {
        (*self).visit(visitor)
    }
}

/// A key value source used by the `log!` macros.
#[doc(hidden)]
pub struct RawKeyValueSource<'a>(pub &'a [(&'a str, &'a dyn ToValue)]);

impl<'a> fmt::Debug for RawKeyValueSource<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RawKeyValueSource").finish()
    }
}

impl<'a> KeyValueSource for RawKeyValueSource<'a> {
    fn visit<'kvs, V>(&'kvs self, visitor: V) -> Result<(), Error>
    where
        V: Visitor<'kvs>
    {
        self.0.visit(visitor)
    }
}

/// A key value source on a `Record`.
#[derive(Clone, Copy)]
pub struct RecordKeyValueSource<'a>(&'a dyn ErasedKeyValueSource);

impl<'a> RecordKeyValueSource<'a> {
    pub(crate) fn erased(kvs: &'a impl KeyValueSource) -> Self {
        RecordKeyValueSource(kvs)
    }
}

impl<'a> fmt::Debug for RecordKeyValueSource<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("KeyValueSource").finish()
    }
}

impl<'a> Default for RecordKeyValueSource<'a> {
    fn default() -> Self {
        RecordKeyValueSource(&RawKeyValueSource(&[]))
    }
}

impl<'a> KeyValueSource for RecordKeyValueSource<'a> {
    fn visit<'kvs, V>(&'kvs self, mut visitor: V) -> Result<(), Error>
    where
        V: Visitor<'kvs>
    {
        self.0.erased_visit(&mut visitor)
    }
}

/// A trait that erases a `KeyValueSource` so it can be stored
/// in a `Record` without requiring any generic parameters.
trait ErasedKeyValueSource {
    fn erased_visit<'kvs, 'vis>(&'kvs self, visitor: &'vis mut dyn Visitor<'kvs>) -> Result<(), Error>;
}

impl<KVS> ErasedKeyValueSource for KVS
where
    KVS: KeyValueSource + ?Sized,
{
    fn erased_visit<'kvs, 'vis>(&'kvs self, visitor: &'vis mut dyn Visitor<'kvs>) -> Result<(), Error> {
        self.visit(visitor)
    }
}
