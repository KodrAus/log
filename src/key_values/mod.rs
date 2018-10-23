//! Structured key-value pairs for log records.

#[macro_use]
mod macros;

mod error;
pub mod value;
pub mod source;

pub use self::error::Error;

#[doc(inline)]
pub use self::source::Source;

mod private {
    use std::fmt;
    use std::cmp::Ordering;
    use std::hash::{Hash, Hasher};
    use std::borrow::Borrow;

    use super::value;

    /// The key in a key-value pair.
    /// 
    /// The `Key` type abstracts over owned or borrowed keys.
    pub struct Key<'k>(KeyInner<'k>);

    // pub(super)
    pub enum KeyInner<'k> {
        Borrowed(&'k str),
        #[cfg(feature = "std")]
        Owned(String),
    }

    impl<'k> Key<'k> {
        pub fn new(k: &'k (impl Borrow<str> + ?Sized)) -> Self {
            Key(KeyInner::Borrowed(k.borrow()))
        }

        pub fn as_str(&self) -> &str {
            match self.0 {
                KeyInner::Borrowed(k) => k,
                #[cfg(feature = "std")]
                KeyInner::Owned(ref k) => &*k,
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
            Key(KeyInner::Borrowed(k))
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

    /// The value in a key-value pair.
    pub struct Value<'v>(ValueInner<'v>);

    // pub(super)
    pub enum ValueInner<'v> {
        Borrowed(&'v dyn value::Value),
        #[cfg(feature = "std")]
        Owned(Box<dyn value::Value>),
    }

    impl<'v> Value<'v> {
        pub fn new(v: &'v impl value::Value) -> Self {
            Value(ValueInner::Borrowed(v))
        }
    }

    impl<'v> fmt::Debug for Value<'v> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self.0 {
                ValueInner::Borrowed(v) => v.fmt(f),
                #[cfg(feature = "std")]
                ValueInner::Owned(ref v) => v.fmt(f),
            }
        }
    }

    // pub(super)
    #[cfg(not(feature = "structured_serde"))]
    pub fn value_inner<'a, 'b>(v: &'a Value<'b>) -> &'a ValueInner<'b> { &v.0 }

    #[cfg(feature = "std")]
    mod std_support {
        use super::*;

        impl<'k> Key<'k> {
            pub fn from_owned(k: impl Into<String>) -> Self {
                Key(KeyInner::Owned(k.into()))
            }
        }

        impl<'k> From<String> for Key<'k> {
            fn from(k: String) -> Self {
                Key::from_owned(k)
            }
        }

        impl<'v> Value<'v> {
            pub fn from_owned(v: impl value::Value + 'static) -> Self {
                Value(ValueInner::Owned(Box::new(v)))
            }
        }
    }

    #[cfg(feature = "structured_serde")]
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

        impl<'v> Serialize for Value<'v> {
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
}