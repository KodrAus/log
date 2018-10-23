//! Sources of key-value pairs.

use std::fmt;
use std::borrow::Borrow;
use std::marker::PhantomData;

use super::{Key, Value, Error};

/// A source for key value pairs that can be serialized.
pub trait Source {
    /// Serialize the key value pairs.
    fn visit<'kvs>(&'kvs self, visitor: &mut dyn SourceVisitor<'kvs>) -> Result<(), Error>;

    /// Erase this `Source` so it can be used without
    /// requiring generic type parameters.
    fn erase(&self) -> ErasedSource
    where
        Self: Sized,
    {
        ErasedSource::erased(self)
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
    fn get<'kvs, Q>(&'kvs self, key: Q) -> Option<&'kvs dyn Value>
    where
        Q: Borrow<str>,
    {
        struct Get<'k, 'v>(Key<'k>, Option<&'v dyn Value>);

        impl<'k, 'kvs> SourceVisitor<'kvs> for Get<'k, 'kvs> {
            fn visit_pair(&mut self, k: Key<'kvs>, v: &'kvs dyn Value) -> Result<(), Error> {
                if k == self.0 {
                    self.1 = Some(v);
                }

                Ok(())
            }
        }

        let mut visitor = Get(Key::from_borrow(&key), None);
        let _ = self.visit(&mut visitor);

        visitor.1
    }

    /// An adapter to borrow self.
    fn by_ref(&self) -> &Self {
        self
    }

    /// Chain two `Source`s together.
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
        F: FnMut(Key, &dyn Value) -> Result<(), E>,
        E: Into<Error>,
    {
        struct ForEach<F, E>(F, PhantomData<E>);

        impl<'kvs, F, E> SourceVisitor<'kvs> for ForEach<F, E>
        where
            F: FnMut(Key, &dyn Value) -> Result<(), E>,
            E: Into<Error>,
        {
            fn visit_pair(&mut self, k: Key<'kvs>, v: &'kvs dyn Value) -> Result<(), Error> {
                (self.0)(k, v).map_err(Into::into)
            }
        }

        self.visit(&mut ForEach(f, Default::default()))
    }

    /// Serialize the key-value pairs as a map.
    #[cfg(feature = "structured_serde")]
    fn serialize_as_map(self) -> SerializeAsMap<Self>
    where
        Self: Sized,
    {
        SerializeAsMap(self)
    }

    /// Serialize the key-value pairs as a sequence.
    #[cfg(feature = "structured_serde")]
    fn serialize_as_seq(self) -> SerializeAsSeq<Self>
    where
        Self: Sized,
    {
        SerializeAsSeq(self)
    }
}

impl<'a, T: ?Sized> Source for &'a T
where
    T: Source,
{
    fn visit<'kvs>(&'kvs self, visitor: &mut dyn SourceVisitor<'kvs>) -> Result<(), Error> {
        (*self).visit(visitor)
    }

    fn get<'kvs, Q>(&'kvs self, key: Q) -> Option<&'kvs dyn Value>
    where
        Q: Borrow<str>,
    {
        (*self).get(key)
    }
}

/// A visitor for key value pairs.
/// 
/// The lifetime of the keys and values is captured by the `'kvs` type.
pub trait SourceVisitor<'kvs> {
    /// Visit a key value pair.
    fn visit_pair(&mut self, k: Key<'kvs>, v: &'kvs dyn Value) -> Result<(), Error>;
}

impl<'a, 'kvs, T: ?Sized> SourceVisitor<'kvs> for &'a mut T
where
    T: SourceVisitor<'kvs>,
{
    fn visit_pair(&mut self, k: Key<'kvs>, v: &'kvs dyn Value) -> Result<(), Error> {
        (*self).visit_pair(k, v)
    }
}

/// A chain of two `Source`s.
#[derive(Debug)]
pub struct Chained<A, B>(A, B);

impl<A, B> Source for Chained<A, B>
where
    A: Source,
    B: Source,
{
    fn visit<'kvs>(&'kvs self, visitor: &mut dyn SourceVisitor<'kvs>) -> Result<(), Error> {
        self.0.visit(visitor)?;
        self.1.visit(visitor)?;

        Ok(())
    }
}

/// Serialize the key-value pairs as a map.
#[derive(Debug)]
#[cfg(feature = "structured_serde")]
pub struct SerializeAsMap<KVS>(KVS);

/// Serialize the key-value pairs as a sequence.
#[derive(Debug)]
#[cfg(feature = "structured_serde")]
pub struct SerializeAsSeq<KVS>(KVS);

impl<K, V> Source for (K, V)
where
    K: Borrow<str>,
    V: Value,
{
    fn visit<'kvs>(&'kvs self, visitor: &mut dyn SourceVisitor<'kvs>) -> Result<(), Error>
    {
        visitor.visit_pair(Key::from_borrow(&self.0), &self.1)
    }
}

impl<KVS> Source for [KVS] where KVS: Source {
    fn visit<'kvs>(&'kvs self, visitor: &mut dyn SourceVisitor<'kvs>) -> Result<(), Error> {
        for kv in self {
            kv.visit(visitor)?;
        }

        Ok(())
    }
}

/// A key value source on a `Record`.
#[derive(Clone)]
pub struct ErasedSource<'a>(&'a dyn ErasedSourceBridge);

impl<'a> ErasedSource<'a> {
    pub fn erased(kvs: &'a impl Source) -> Self {
        ErasedSource(kvs)
    }

    pub fn empty() -> Self {
        ErasedSource(&(&[] as &[(&str, &dyn Value)]))
    }
}

impl<'a> fmt::Debug for ErasedSource<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Source").finish()
    }
}

impl<'a> Source for ErasedSource<'a> {
    fn visit<'kvs>(&'kvs self, visitor: &mut SourceVisitor<'kvs>) -> Result<(), Error> {
        self.0.erased_visit(visitor)
    }

    fn get<'kvs, Q>(&'kvs self, key: Q) -> Option<&'kvs dyn Value>
    where
        Q: Borrow<str>,
    {
        self.0.erased_get(key.borrow())
    }
}

/// A trait that erases a `Source` so it can be stored
/// in a `Record` without requiring any generic parameters.
trait ErasedSourceBridge {
    fn erased_visit<'kvs>(&'kvs self, visitor: &mut dyn SourceVisitor<'kvs>) -> Result<(), Error>;
    fn erased_get<'kvs>(&'kvs self, key: &str) -> Option<&'kvs dyn Value>;
}

impl<KVS> ErasedSourceBridge for KVS
where
    KVS: Source + ?Sized,
{
    fn erased_visit<'kvs>(&'kvs self, visitor: &mut dyn SourceVisitor<'kvs>) -> Result<(), Error> {
        self.visit(visitor)
    }

    fn erased_get<'kvs>(&'kvs self, key: &str) -> Option<&'kvs dyn Value> {
        self.get(key)
    }
}

#[cfg(feature = "structured_serde")]
mod serde_support {
    use super::*;

    use serde::ser::{Serialize, Serializer, SerializeMap, SerializeSeq};

    impl<KVS> Serialize for SerializeAsMap<KVS>
    where
        KVS: Source,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut map = serializer.serialize_map(None)?;

            self.0
                .by_ref()
                .try_for_each(|k, v| map.serialize_entry(&k, &v))
                .map_err(Error::into_serde)?;

            map.end()
        }
    }

    impl<KVS> Serialize for SerializeAsSeq<KVS>
    where
        KVS: Source,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut seq = serializer.serialize_seq(None)?;

            self.0
                .by_ref()
                .try_for_each(|k, v| seq.serialize_element(&(&k, &v)))
                .map_err(Error::into_serde)?;

            seq.end()
        }
    }
}

#[cfg(feature = "structured_serde")]
pub use self::serde_support::*;

#[cfg(feature = "std")]
mod std_support {
    use super::*;

    use std::hash::Hash;
    use std::collections::{HashMap, BTreeMap};

    impl<KVS> Source for Vec<KVS> where KVS: Source {
        fn visit<'kvs>(&'kvs self, visitor: &mut dyn SourceVisitor<'kvs>) -> Result<(), Error> {
            self.as_slice().visit(visitor)
        }
    }

    impl<K, V> Source for BTreeMap<K, V>
    where
        K: Borrow<str> + Ord,
        V: Value,
    {
        fn visit<'kvs>(&'kvs self, visitor: &mut dyn SourceVisitor<'kvs>) -> Result<(), Error>
        {
            for (k, v) in self {
                visitor.visit_pair(Key::from_borrow(k), &*v)?;
            }

            Ok(())
        }

        fn get<'kvs, Q>(&'kvs self, key: Q) -> Option<&'kvs dyn Value>
        where
            Q: Borrow<str>,
        {
            BTreeMap::get(self, key.borrow()).map(|v| v as &dyn Value)
        }
    }

    impl<K, V> Source for HashMap<K, V>
    where
        K: Borrow<str> + Eq + Hash,
        V: Value,
    {
        fn visit<'kvs>(&'kvs self, visitor: &mut dyn SourceVisitor<'kvs>) -> Result<(), Error>
        {
            for (k, v) in self {
                visitor.visit_pair(Key::from_borrow(k), &*v)?;
            }

            Ok(())
        }

        fn get<'kvs, Q>(&'kvs self, key: Q) -> Option<&'kvs dyn Value>
        where
            Q: Borrow<str>,
        {
            HashMap::get(self, key.borrow()).map(|v| v as &dyn Value)
        }
    }
}

#[cfg(feature = "std")]
pub use self::std_support::*;
