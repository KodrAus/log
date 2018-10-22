use std::fmt;
use std::cmp::Ordering;

/// The key in a key-value pair.
pub trait Key {
    fn to_str(&self) -> &str;
}

impl<'a, T: ?Sized> Key for &'a T
where
    T: Key,
{
    fn to_str(&self) -> &str {
        (**self).to_str()
    }
}

impl<'a> PartialEq for &'a dyn Key {
    fn eq(&self, other: &Self) -> bool {
        self.to_str().eq(other.to_str())
    }
}

impl<'a> Eq for &'a dyn Key { }

impl<'a> PartialOrd for &'a dyn Key {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.to_str().partial_cmp(other.to_str())
    }
}

impl<'a> Ord for &'a dyn Key {
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_str().cmp(other.to_str())
    }
}

impl<'a> fmt::Debug for &'a dyn Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.to_str().fmt(f)
    }
}

impl<'a> fmt::Display for &'a dyn Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.to_str().fmt(f)
    }
}

impl Key for str {
    fn to_str(&self) -> &str {
        self
    }
}

#[cfg(feature = "structured_serde")]
mod serde_support {
    use super::*;

    use serde::{Serialize, Serializer};

    impl<'a> Serialize for dyn Key + 'a {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            self.to_str().serialize(serializer)
        }
    }
}

#[cfg(feature = "structured_serde")]
pub use self::serde_support::*;

#[cfg(feature = "std")]
mod std_support {
    use super::*;

    use std::hash::{Hash, Hasher};

    impl<'a> Hash for &'a dyn Key {
        fn hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.to_str().hash(state)
        }
    }

    impl Key for String {
        fn to_str(&self) -> &str {
            &*self
        }
    }
}

#[cfg(feature = "std")]
pub use self::std_support::*;
