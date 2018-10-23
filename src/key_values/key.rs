use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::borrow::Borrow;

pub struct Key<'k>(KeyInner<'k>);

enum KeyInner<'k> {
    Borrowed(&'k str),
    #[cfg(feature = "std")]
    Owned(String),
}

impl<'k> Key<'k> {
    pub fn from_borrow(k: &'k impl Borrow<str>) -> Self {
        Key(KeyInner::Borrowed(k.borrow()))
    }

    #[cfg(feature = "std")]
    pub fn from_owned(k: impl Into<String>) -> Self {
        Key(KeyInner::Owned(k.into()))
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
