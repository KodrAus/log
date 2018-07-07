use std::fmt;
use serde;

/// Converting into a `Key`.
pub trait ToKey {
    /// Perform the conversion.
    fn to_key(&self) -> Key;
}

impl<T: fmt::Display> ToKey for T {
    fn to_key(&self) -> Key {
        Key::from_display(self)
    }
}

impl<'a> ToKey for &'a dyn ToKey {
    fn to_key(&self) -> Key {
        (*self).to_key()
    }
}

/// A single property key.
/// 
/// `Key`s borrow their underlying data and implement `serde::Serialize` and `fmt::Display`.
#[derive(Clone, Copy)]
#[allow(missing_debug_implementations)]
pub struct Key<'a> {
    inner: KeyInner<'a>,
}

#[derive(Clone, Copy)]
enum KeyInner<'a> {
    Display(&'a dyn fmt::Display),
}

impl<'a> Key<'a> {
    /// Get a `Key` from some displayable reference.
    pub fn from_display(key: &'a impl fmt::Display) -> Self {
        Key {
            inner: KeyInner::Display(key),
        }
    }
}

impl<'a> serde::Serialize for Key<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.inner {
            KeyInner::Display(v) => serializer.collect_str(&v),
        }
    }
}

impl<'a> fmt::Display for Key<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.inner {
            KeyInner::Display(k) => write!(f, "{}", k)
        }
    }
}
