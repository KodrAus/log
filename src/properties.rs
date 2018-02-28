//! Log record properties.

use std::fmt;

use serde;

pub use erased_serde::Serialize;

pub trait Serializer {
    fn serialize_entry(&mut self, key: &str, value: &Serialize);
}

pub trait KeyValues {
    fn serialize(&self, serializer: &mut Serializer);
}

#[doc(hidden)]
pub struct RawKeyValues<'a>(pub &'a [(&'a str, &'a Serialize)]);

impl<'a> fmt::Debug for RawKeyValues<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RawKeyValues").finish()
    }
}

impl<T, K, V> KeyValues for T
where
    for<'a> &'a T: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: serde::Serialize
{
    fn serialize(&self, serializer: &mut Serializer) {
        for (key, value) in self.into_iter() {
            serializer.serialize_entry(key.as_ref(), &value);
        }
    }
}

impl<T> Serializer for T
where
    T: serde::ser::SerializeMap
{
    fn serialize_entry(&mut self, key: &str, value: &Serialize) {
        let _ = serde::ser::SerializeMap::serialize_entry(self, key, value);
    }
}

impl<'a> KeyValues for RawKeyValues<'a> {
    fn serialize(&self, serializer: &mut Serializer) {
        for &(key, value) in self.0.iter() {
            serializer.serialize_entry(key.as_ref(), &value);
        }
    }
}

/// A chain of properties.
#[derive(Clone)]
pub struct Properties<'a> {
    kvs: &'a KeyValues,
    parent: Option<&'a Properties<'a>>,
}

impl<'a> Properties<'a> {
    pub(crate) fn root(properties: &'a KeyValues) -> Self {
        Properties {
            kvs: properties,
            parent: None
        }
    }

    pub(crate) fn chained(properties: &'a KeyValues, parent: &'a Properties) -> Self {
        Properties {
            kvs: properties,
            parent: Some(parent)
        }
    }
}

impl<'a> KeyValues for Properties<'a> {
    fn serialize(&self, serializer: &mut Serializer) {
        self.kvs.serialize(serializer);

        if let Some(parent) = self.parent {
            parent.serialize(serializer);
        }
    }
}

impl<'a> serde::Serialize for Properties<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        use serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(None)?;

        KeyValues::serialize(self, &mut map);

        map.end()
    }
}

impl<'a> fmt::Debug for Properties<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Properties").finish()
    }
}

impl<'a> Default for Properties<'a> {
    fn default() -> Self {
        Properties {
            kvs: &RawKeyValues(&[]),
            parent: None,
        }
    }
}
