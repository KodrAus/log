//! Structured keys.

use std::fmt;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::borrow::Borrow;

/// A type that can be converted into a key.
pub trait ToKey {
    /// Perform the conversion.
    fn to_key(&self) -> Key;
}

impl<'a, T: ?Sized> ToKey for &'a T
where
    T: ToKey,
{
    fn to_key(&self) -> Key {
        (**self).to_key()
    }
}

impl ToKey for str {
    fn to_key(&self) -> Key {
        Key::from_str(self, None)
    }
}

impl<'k> ToKey for Key<'k> {
    fn to_key(&self) -> Key {
        Key::from_str(self, self.index)
    }
}

/// The key in a key-value pair.
/// 
/// The `Key` type abstracts over owned or borrowed keys.
pub struct Key<'k> {
    inner: Str<'k>,
    index: Option<usize>,
}

impl<'k> Key<'k> {
    /// Create a key from a borrowed string and optional index.
    pub fn from_str(key: &'k (impl Borrow<str> + ?Sized), index: Option<usize>) -> Self {
        Key {
            inner: Str::Borrowed(key.borrow()),
            index,
        }
    }

    pub fn index(&self) -> Option<usize> {
        self.index
    }

    pub fn as_str(&self) -> &str {
        match self.inner {
            Str::Borrowed(k) => k,
            #[cfg(feature = "std")]
            Str::Owned(ref k) => &*k,
        }
    }
}

impl<'k> AsRef<str> for Key<'k> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'k> Borrow<str> for Key<'k> {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<'k> From<&'k str> for Key<'k> {
    fn from(k: &'k str) -> Self {
        Key::from_str(k, None)
    }
}

impl<'k> PartialEq for Key<'k> {
    fn eq(&self, other: &Self) -> bool {
        self.as_str().eq(other.as_str())
    }
}

impl<'k> Eq for Key<'k> {}

impl<'k> PartialOrd for Key<'k> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl<'k> Ord for Key<'k> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl<'k> Hash for Key<'k> {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.as_str().hash(state)
    }
}

impl<'k> fmt::Debug for Key<'k> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl<'k> fmt::Display for Key<'k> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

enum Str<'k> {
    Borrowed(&'k str),
    #[cfg(feature = "std")]
    Owned(String),
}

#[cfg(feature = "std")]
mod std_support {
    use super::*;

    impl<'k> Key<'k> {
        /// Create a key from an owned string and optional index.
        pub fn from_owned(key: impl Into<String>, index: Option<usize>) -> Self {
            Key {
                inner: Str::Owned(key.into()),
                index,
            }
        }
    }

    impl ToKey for String {
        fn to_key(&self) -> Key {
            Key::from_str(self, None)
        }
    }

    impl<'k> From<String> for Key<'k> {
        fn from(k: String) -> Self {
            Key::from_owned(k, None)
        }
    }
}

#[cfg(feature = "kv_sval")]
mod sval_support {
    use super::*;

    use sval::value::{self, Value};

    impl<'k> Value for Key<'k> {
        fn stream(&self, stream: &mut value::Stream) -> Result<(), value::Error> {
            self.as_str().stream(stream)
        }
    }
}

#[cfg(feature = "kv_serde")]
mod serde_support {
    use super::*;

    use serde::{Serialize, Serializer};

    impl<'k> Serialize for Key<'k> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            self.as_str().serialize(serializer)
        }
    }
}