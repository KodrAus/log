//! Object-safe iteration for key-value pairs.

use std::fmt;
use std::collections::BTreeMap;
use std::borrow::Borrow;
use std::collections::Bound;

pub use erased_serde::Serialize;

/// A key into a set of properties.
/// 
/// The key can either be an index or a reference to the last seen key.
#[derive(Debug, Clone, Copy)]
pub struct Key<'a>(KeyInner<'a>);

#[derive(Debug, Clone, Copy)]
enum KeyInner<'a> {
    /// An index key.
    Number(u64),
    /// A reference to another key.
    String(&'a str)
}

impl<'a> From<u64> for Key<'a> {
    fn from(key: u64) -> Self {
        Key(KeyInner::Number(key))
    }
}

impl<'a> From<&'a str> for Key<'a> {
    fn from(key: &'a str) -> Self {
        Key(KeyInner::String(key))
    }
}

impl<'a> Key<'a> {
    /// Create a key from a `u64` index.
    pub fn from_u64(key: u64) -> Self {
        Self::from(key)
    }

    /// Create a key from a borrowed string.
    pub fn from_str(key: &'a str) -> Self {
        Self::from(key)
    }

    /// Get the key value as a `u64` index.
    pub fn as_u64(&self) -> Option<u64> {
        match self.0 {
            KeyInner::Number(n) => Some(n),
            _ => None
        }
    }

    /// Get the key value as a borrowed string.
    pub fn as_str(&self) -> Option<&str> {
        match self.0 {
            KeyInner::String(s) => Some(s.as_ref()),
            _ => None
        }
    }
}

/// An entry within a key value set.
#[derive(Clone, Copy)]
pub struct Entry<'a> {
    key: &'a str,
    value: &'a Serialize,
    next: Option<Key<'a>>,
}

impl<'a> fmt::Debug for Entry<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Entry").finish()
    }
}

impl<'a> Entry<'a> {
    /// Create an entry from a key, value and optional next key.
    /// 
    /// It's important that the next key follows the current one,
    /// and that each key is only seen once.
    pub fn new<T>(key: &'a str, value: &'a Serialize, next: T) -> Self
    where
        T: Into<Option<Key<'a>>>
    {
        Entry {
            key,
            value,
            next: next.into(),
        }
    }
}

/// A set of key value pairs that can be iterated though.
pub trait KeyValues {
    /// The first entry in the key value set.
    fn first(&self) -> Option<Entry>;
    /// A given entry in the key value set.
    fn entry(&self, key: &Key) -> Option<Entry>;
}

impl<'a> IntoIterator for &'a KeyValues {
    type Item = (&'a str, &'a Serialize);
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            current: self.first(),
            kvs: self
        }
    }
}

impl<'a, T: ?Sized> KeyValues for &'a T where T: KeyValues {
    /// The first entry in the key value set.
    fn first(&self) -> Option<Entry> {
        (*self).first()
    }

    /// A given entry in the key value set.
    fn entry(&self, key: &Key) -> Option<Entry> {
        (*self).entry(key)
    }
}

/// An iterator over key value pairs.
#[derive(Clone, Copy)]
pub struct Iter<'a> {
    current: Option<Entry<'a>>,
    kvs: &'a KeyValues,
}

impl<'a> fmt::Debug for Iter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Iter").finish()
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a str, &'a Serialize);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(Entry { key, value, next, .. }) = self.current.take() {
            let next = next.and_then(|ref key| self.kvs.entry(key));
            self.current = next;

            Some((key, value))
        }
        else {
            None
        }
    }
}

#[doc(hidden)]
pub struct RawKeyValues<'a>(pub &'a [(&'a str, &'a Serialize)]);

impl<'a> fmt::Debug for RawKeyValues<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RawKeyValues").finish()
    }
}

impl<'a, K, V> KeyValues for [(K, V)]
where
    K: Borrow<str>,
    V: Serialize,
{
    fn first(&self) -> Option<Entry> {
        self.entry(&Key::from(0))
    }
    
    fn entry(&self, key: &Key) -> Option<Entry> {
        key.as_u64().and_then(|n| {
            match self.get(n as usize) {
                Some(&(ref k, ref v)) => Some(Entry::new(k.borrow(), v, Key::from(n + 1))),
                None => None
            }
        })
    }
}

impl<K, V> KeyValues for Vec<(K, V)>
where
    K: Borrow<str>,
    V: Serialize,
{
    fn first(&self) -> Option<Entry> {
        KeyValues::first(self.as_slice())
    }

    fn entry(&self, key: &Key) -> Option<Entry> {
        self.as_slice().entry(key)
    }
}

impl<'a> KeyValues for RawKeyValues<'a> {
    fn first(&self) -> Option<Entry> {
        KeyValues::first(&self.0)
    }
    
    fn entry(&self, key: &Key) -> Option<Entry> {
        self.0.entry(key)
    }
}

impl<K, V> KeyValues for BTreeMap<K, V>
where
    K: Borrow<str> + Ord,
    V: Serialize,
{
    fn first(&self) -> Option<Entry> {
        self.keys()
            .next()
            .and_then(|k| self.entry(&Key::from(k.borrow())))
    }

    fn entry(&self, key: &Key) -> Option<Entry> {
        key.as_str().and_then(|s| {
            let mut range = self.range((Bound::Included(s.as_ref()), Bound::Unbounded));
            
            let current = range.next();
            let next = range.next();

            current.map(|(k, v)| {
                Entry::new(k.borrow(), v, next.map(|(k, _)| Key::from(k.borrow())))
            })
        })
    }
}
