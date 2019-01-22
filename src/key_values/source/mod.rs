//! Sources of structured key-value pairs.

mod erased;
mod impls;

use std::marker::PhantomData;

use super::key::ToKey;
use super::value::ToValue;

pub use self::erased::ErasedSource;
pub use super::key::Key;
pub use super::value::Value;

#[doc(inline)]
pub use super::Error;

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
pub trait Source {
    /// Serialize the key value pairs.
    fn visit<'kvs>(&'kvs self, visitor: &mut dyn Visitor<'kvs>) -> Result<(), Error>;

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
    /// If the key is present multiple times, whether or not this
    /// method will return the first or last value for a given key
    /// is not defined.
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
        F: FnMut(Key, Value) -> Result<(), E>,
        E: Into<Error>,
    {
        struct ForEach<F, E>(F, PhantomData<E>);

        impl<'kvs, F, E> Visitor<'kvs> for ForEach<F, E>
        where
            F: FnMut(Key, Value) -> Result<(), E>,
            E: Into<Error>,
        {
            fn visit_pair(&mut self, k: Key<'kvs>, v: Value<'kvs>) -> Result<(), Error> {
                (self.0)(k, v).map_err(Into::into)
            }
        }

        let mut for_each = ForEach(f, Default::default());
        self.visit(&mut for_each)
    }

    /// Serialize the key-value pairs as a map.
    #[cfg(any(feature = "kv_serde", feature = "kv_sval"))]
    fn as_map(self) -> AsMap<Self>
    where
        Self: Sized,
    {
        AsMap(self)
    }

    /// Serialize the key-value pairs as a sequence.
    #[cfg(any(feature = "kv_serde", feature = "kv_sval"))]
    fn as_seq(self) -> AsSeq<Self>
    where
        Self: Sized,
    {
        AsSeq(self)
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
    fn visit<'kvs>(&'kvs self, visitor: &mut dyn Visitor<'kvs>) -> Result<(), Error> {
        self.0.visit(visitor)?;
        self.1.visit(visitor)?;

        Ok(())
    }
}

/// Serialize the key-value pairs as a map.
#[derive(Debug)]
#[cfg(any(feature = "kv_serde", feature = "kv_sval"))]
pub struct AsMap<KVS>(KVS);

/// Serialize the key-value pairs as a sequence.
#[derive(Debug)]
#[cfg(any(feature = "kv_serde", feature = "kv_sval"))]
pub struct AsSeq<KVS>(KVS);

#[cfg(feature = "kv_sval")]
mod sval_support {
    use super::*;

    use sval::value::{self, Value};

    impl<KVS> Value for AsMap<KVS>
    where
        KVS: Source,
    {
        fn stream(&self, stream: &mut value::Stream) -> Result<(), value::Error> {
            stream.map_begin(None)?;

            self.0
                .by_ref()
                .try_for_each(|k, v| {
                    stream.map_key(k)?;
                    stream.map_value(v)
                })?;

            stream.map_end()
        }
    }

    impl<KVS> Value for AsSeq<KVS>
    where
        KVS: Source,
    {
        fn stream(&self, stream: &mut value::Stream) -> Result<(), value::Error> {
            stream.seq_begin(None)?;

            self.0
                .by_ref()
                .try_for_each(|k, v| stream.seq_elem((k, v)))?;

            stream.seq_end()
        }
    }
}

#[cfg(feature = "kv_serde")]
mod serde_support {
    use super::*;

    use serde::ser::{Serialize, Serializer, SerializeMap, SerializeSeq};

    impl<KVS> Serialize for AsMap<KVS>
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
                .try_for_each(|k, v| map.serialize_entry(&k, &v).map_err(Error::from_serde))
                .map_err(Error::into_serde)?;

            map.end()
        }
    }

    impl<KVS> Serialize for AsSeq<KVS>
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
                .try_for_each(|k, v| seq.serialize_element(&(&k, &v)).map_err(Error::from_serde))
                .map_err(Error::into_serde)?;

            seq.end()
        }
    }
}
