//! Log property keys.
//! 
//! This module contains the `Key` type,
//! which is roughly a `dyn AsRef<str>`.

use std::fmt;
#[cfg(feature = "std")]
use std::borrow;
use serde;

/// Converting into a `Key`.
pub trait ToKey {
    /// Perform the conversion.
    fn to_key(&self) -> Key;
}

impl ToKey for str {
    fn to_key(&self) -> Key {
        Key::from_str(self)
    }
}

#[cfg(feature = "std")]
impl ToKey for String {
    fn to_key(&self) -> Key {
        Key::from_str(self)
    }
}

#[cfg(feature = "std")]
impl<'a> ToKey for borrow::Cow<'a, str> {
    fn to_key(&self) -> Key {
        Key::from_str(self.as_ref())
    }
}

impl<'a, K: ?Sized> ToKey for &'a K
where
    K: ToKey,
{
    fn to_key(&self) -> Key {
        (*self).to_key()
    }
}

/// A single property key.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Key<'kvs> {
    inner: &'kvs str,
}

impl<'kvs> ToKey for Key<'kvs> {
    fn to_key(&self) -> Key {
        Key { inner: self.inner }
    }
}

impl<'kvs> Key<'kvs> {
    /// Get a `Key` from a borrowed string.
    pub fn from_str(key: &'kvs (impl AsRef<str> + ?Sized)) -> Self {
        Key {
            inner: key.as_ref(),
        }
    }

    /// Get a borrowed string from a `Key`.
    pub fn as_str(&self) -> &str {
        &self.inner
    }
}

impl<'kvs> AsRef<str> for Key<'kvs> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[cfg(feature = "std")]
impl<'kvs> borrow::Borrow<str> for Key<'kvs> {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<'kvs> serde::Serialize for Key<'kvs> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.inner)
    }
}

impl<'kvs> fmt::Display for Key<'kvs> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl<'kvs> fmt::Debug for Key<'kvs> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner.fmt(f)
    }
}
