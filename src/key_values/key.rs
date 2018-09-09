//! Log property keys.
//! 
//! This module contains the `Key` type,
//! which is roughly a `dyn AsRef<str>`.

use std::{fmt, cmp, hash};
#[cfg(feature = "std")]
use std::borrow;
use serde;

/// Converting into a `Key`.
pub trait ToKey {
    /// Perform the conversion.
    fn to_key(&self) -> Key;
}

impl<T: AsRef<str>> ToKey for T {
    fn to_key(&self) -> Key {
        Key::from_str(self)
    }
}

impl<'a> ToKey for &'a dyn ToKey {
    fn to_key(&self) -> Key {
        (*self).to_key()
    }
}

/// A single property key.
/// 
/// `Key`s borrow their underlying data and can be treated as strings.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Key<'a> {
    inner: KeyInner<'a>,
}

#[derive(Clone, Copy)]
enum KeyInner<'a> {
    Borrowed(&'a str),
}

impl<'a> KeyInner<'a> {
    fn as_str(&self) -> &str {
        match *self {
            KeyInner::Borrowed(k) => k,
        }
    }
}

impl<'a> Key<'a> {
    /// Get a `Key` from a borrowed string.
    pub fn from_str(key: &'a impl AsRef<str>) -> Self {
        Key {
            inner: KeyInner::Borrowed(key.as_ref()),
        }
    }

    /// Get a borrowed string from a `Key`.
    pub fn as_str(&self) -> &str {
        self.inner.as_str()
    }
}

impl<'a> serde::Serialize for Key<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'a> fmt::Display for Key<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.inner {
            KeyInner::Borrowed(k) => write!(f, "{}", k)
        }
    }
}

impl<'a> fmt::Debug for Key<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut f = f.debug_struct("Key");

        match self.inner {
            KeyInner::Borrowed(ref k) => f.field("value", k)
        };

        f.finish()
    }
}

impl<'a> AsRef<str> for Key<'a> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[cfg(feature = "std")]
impl<'a> borrow::Borrow<str> for Key<'a> {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<'a> PartialEq for KeyInner<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.as_str().eq(other.as_str())
    }
}

impl<'a> Eq for KeyInner<'a> { }

impl<'a> PartialOrd for KeyInner<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl<'a> Ord for KeyInner<'a> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl<'a> hash::Hash for KeyInner<'a> {
    fn hash<H>(&self, state: &mut H)
    where
        H: hash::Hasher,
    {
        self.as_str().hash(state)
    }
}
