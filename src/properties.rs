//! Log record properties.

#[cfg(feature = "erased-serde")]
mod imp {
    use std::fmt;

    use serde;

    pub use erased_serde::Serialize as Value;

    /// A serializer for key value pairs.
    pub trait Serializer {
        /// Serialize the key and value.
        fn serialize_kv(&mut self, kv: &KeyValue);
    }

    /// A set of key value pairs that can be serialized.
    pub trait KeyValues {
        /// Serialize the key value pairs.
        fn serialize(&self, serializer: &mut Serializer);
    }

    /// A single key value pair.
    pub trait KeyValue {
        /// Get the key.
        fn key(&self) -> &str;
        /// Get the value.
        fn value(&self) -> &Value;
    }

    impl<K, V> KeyValue for (K, V)
    where
        K: AsRef<str>,
        V: serde::Serialize,
    {
        fn key(&self) -> &str {
            self.0.as_ref()
        }

        fn value(&self) -> &Value {
            &self.1
        }
    }

    impl<'a, T: ?Sized> KeyValue for &'a T
    where
        T: KeyValue
    {
        fn key(&self) -> &str {
            (*self).key()
        }

        fn value(&self) -> &Value {
            (*self).value()
        }
    }

    impl<'a, T: ?Sized, KV> KeyValues for &'a T
    where
        &'a T: IntoIterator<Item = KV>,
        KV: KeyValue
    {
        fn serialize(&self, serializer: &mut Serializer) {
            for kv in self.into_iter() {
                serializer.serialize_kv(&kv);
            }
        }
    }

    impl<T> Serializer for T
    where
        T: serde::ser::SerializeMap
    {
        fn serialize_kv(&mut self, kv: &KeyValue) {
            let _ = serde::ser::SerializeMap::serialize_entry(self, kv.key(), kv.value());
        }
    }

    #[doc(hidden)]
    pub struct RawKeyValues<'a>(pub &'a [(&'a str, &'a Value)]);

    impl<'a> fmt::Debug for RawKeyValues<'a> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.debug_struct("RawKeyValues").finish()
        }
    }

    impl<'a> KeyValues for RawKeyValues<'a> {
        fn serialize(&self, serializer: &mut Serializer) {
            self.0.serialize(serializer)
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
}

#[cfg(not(feature = "erased-serde"))]
mod imp {
    use std::fmt;

    /// A chain of properties.
    pub struct Properties<'a> {
        _kvs: &'a (),
        _parent: &'a (),
    }

    impl<'a> fmt::Debug for Properties<'a> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.debug_struct("Properties").finish()
        }
    }
}

pub use self::imp::*;
