use std::fmt;

use super::{Source, ToKey, Key, Value, Visitor, Error};

/// An erased `Source`.
#[derive(Clone, Copy)]
pub struct ErasedSource<'a>(&'a dyn ErasedSourceBridge);

impl<'a> ErasedSource<'a> {
    /// Erase a `Source`.
    pub fn erased(kvs: &'a impl Source) -> Self {
        ErasedSource(kvs)
    }

    /// Erase an empty `Source`.
    pub fn empty() -> Self {
        ErasedSource(&(&[] as &[(&str, Value)]))
    }
}

impl<'a> fmt::Debug for ErasedSource<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Source").finish()
    }
}

impl<'a> Source for ErasedSource<'a> {
    fn visit<'kvs>(&'kvs self, visitor: &mut Visitor<'kvs>) -> Result<(), Error> {
        self.0.erased_visit(visitor)
    }

    fn get<'kvs, Q>(&'kvs self, key: Q) -> Option<Value<'kvs>>
    where
        Q: ToKey,
    {
        self.0.erased_get(key.to_key())
    }
}

/// A trait that erases a `Source` so it can be stored
/// in a `Record` without requiring any generic parameters.
trait ErasedSourceBridge {
    fn erased_visit<'kvs>(&'kvs self, visitor: &mut dyn Visitor<'kvs>) -> Result<(), Error>;
    fn erased_get<'kvs>(&'kvs self, key: Key) -> Option<Value<'kvs>>;
}

impl<KVS> ErasedSourceBridge for KVS
where
    KVS: Source + ?Sized,
{
    fn erased_visit<'kvs>(&'kvs self, visitor: &mut dyn Visitor<'kvs>) -> Result<(), Error> {
        self.visit(visitor)
    }

    fn erased_get<'kvs>(&'kvs self, key: Key) -> Option<Value<'kvs>> {
        self.get(key)
    }
}
