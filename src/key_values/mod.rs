//! Structured key-value pairs for log records.

#[macro_use]
mod macros;

mod error;
mod value;
pub mod source;

pub use self::error::Error;
pub use self::value::{Value, ValueVisitor};

#[doc(inline)]
pub use self::source::{Source, SourceVisitor};

use std::fmt;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::borrow::Borrow;

/// The key in a key-value pair.
/// 
/// The `Key` type abstracts over owned or borrowed keys.
pub struct SourceKey<'k>(KeyInner<'k>);

enum KeyInner<'k> {
    Borrowed(&'k str),
    #[cfg(feature = "std")]
    Owned(String),
}

impl<'k> SourceKey<'k> {
    pub fn new(k: &'k (impl Borrow<str> + ?Sized)) -> Self {
        SourceKey(KeyInner::Borrowed(k.borrow()))
    }

    pub fn as_str(&self) -> &str {
        match self.0 {
            KeyInner::Borrowed(k) => k,
            #[cfg(feature = "std")]
            KeyInner::Owned(ref k) => &*k,
        }
    }
}

impl<'k> AsRef<str> for SourceKey<'k> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'k> Borrow<str> for SourceKey<'k> {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<'k> From<&'k str> for SourceKey<'k> {
    fn from(k: &'k str) -> Self {
        SourceKey(KeyInner::Borrowed(k))
    }
}

impl<'k> PartialEq for SourceKey<'k> {
    fn eq(&self, other: &Self) -> bool {
        self.as_str().eq(other.as_str())
    }
}

impl<'k> Eq for SourceKey<'k> {}

impl<'k> PartialOrd for SourceKey<'k> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl<'k> Ord for SourceKey<'k> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl<'k> Hash for SourceKey<'k> {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.as_str().hash(state)
    }
}

/// The value in a key-value pair.
pub struct SourceValue<'v>(ValueInner<'v>);

enum ValueInner<'v> {
    Borrowed(&'v dyn value::Value),
    #[cfg(feature = "std")]
    Owned(Box<dyn value::Value>),
}

impl<'v> SourceValue<'v> {
    pub fn new(v: &'v impl value::Value) -> Self {
        SourceValue(ValueInner::Borrowed(v))
    }
}

impl<'v> fmt::Debug for SourceValue<'v> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            ValueInner::Borrowed(v) => v.fmt(f),
            #[cfg(feature = "std")]
            ValueInner::Owned(ref v) => v.fmt(f),
        }
    }
}

#[cfg(feature = "std")]
mod std_support {
    use super::*;

    impl<'k> SourceKey<'k> {
        pub fn from_owned(k: impl Into<String>) -> Self {
            SourceKey(KeyInner::Owned(k.into()))
        }
    }

    impl<'k> From<String> for SourceKey<'k> {
        fn from(k: String) -> Self {
            SourceKey::from_owned(k)
        }
    }

    impl<'v> SourceValue<'v> {
        pub fn from_owned(v: impl value::Value + 'static) -> Self {
            SourceValue(ValueInner::Owned(Box::new(v)))
        }
    }
}

#[cfg(feature = "structured_serde")]
mod serde_support {
    use super::*;

    use serde::{Serialize, Serializer};

    impl<'k> Serialize for SourceKey<'k> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            self.as_str().serialize(serializer)
        }
    }

    impl<'v> Serialize for SourceValue<'v> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match self.0 {
                ValueInner::Borrowed(v) => v.serialize(serializer),
                ValueInner::Owned(ref v) => v.serialize(serializer),
            }
        }
    }
}
