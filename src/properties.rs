use std::fmt;
use std::collections::BTreeMap;
use std::borrow::Borrow;
use std::collections::Bound;

pub use erased_serde::Serialize;

// TODO: Can this be a trait borrowed from KeyValues?
#[derive(Debug)]
pub enum Key<'a> {
    Number(u64),
    String(&'a str)
}

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
    pub fn new(key: &'a str, value: &'a Serialize, next: Option<Key<'a>>) -> Self {
        Entry {
            key,
            value,
            next,
        }
    }
}

pub trait KeyValues {
    fn first(&self) -> Option<Entry>;
    fn entry(&self, key: &Key) -> Option<Entry>;
}

impl<'a, K, V> KeyValues for [(K, V)]
where
    K: Borrow<str>,
    V: Serialize,
{
    fn first(&self) -> Option<Entry> {
        self.entry(&Key::Number(0))
    }
    
    fn entry(&self, key: &Key) -> Option<Entry> {
        match *key {
            Key::Number(n) => {
                match self.get(n as usize) {
                    Some(&(ref k, ref v)) => Some(Entry::new(k.borrow(), v, Some(Key::Number(n + 1)))),
                    None => None
                }
            },
            Key::String(_) => None
        }
    }
}

impl<'a, K, V> KeyValues for &'a [(K, V)]
where
    K: Borrow<str>,
    V: Serialize,
{
    fn first(&self) -> Option<Entry> {
        KeyValues::first(*self)
    }
    
    fn entry(&self, key: &Key) -> Option<Entry> {
        (*self).entry(key)
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

pub struct RawKeyValues<'a>(pub &'a [(&'a str, &'a Serialize)]);

impl<'a> fmt::Debug for RawKeyValues<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("RawKeyValues").finish()
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
            .and_then(|k| self.entry(&Key::String(k.borrow())))
    }

    fn entry(&self, key: &Key) -> Option<Entry> {
        match *key {
            Key::String(s) => {
                let mut range = self.range((Bound::Included(s), Bound::Unbounded));
                
                let current = range.next();
                let next = range.next();
                
                current.map(|(k, v)| {
                    Entry::new(k.borrow(), v as &Serialize, next.map(|(k, _)| Key::String(k.borrow())))
                })
            },
            Key::Number(_) => None
        }
    }
}

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

impl<'a> fmt::Debug for Properties<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut d = f.debug_set();

        for (ref k, _) in self {
            d.entry(k);
        }

        d.finish()
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

struct KeyValuesIter<'a> {
    current: Option<Entry<'a>>,
    kvs: &'a KeyValues,
}

impl<'a> KeyValuesIter<'a> {
    fn over(kvs: &'a KeyValues) -> Self {
        KeyValuesIter {
            current: kvs.first(),
            kvs
        }
    }
}

impl<'a> Iterator for KeyValuesIter<'a> {
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

/// An iterator over properties.
/// 
/// Properties aren't guaranteed to be unique (the same key may be repeated with different values).
/// Properties also aren't guaranteed to be ordered.
pub struct Iter<'a, 'b> where 'a: 'b {
    properties: &'b Properties<'a>,
    iter: KeyValuesIter<'a>,
}

impl<'a, 'b> fmt::Debug for Iter<'a, 'b> where 'a: 'b {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Iter").finish()
    }
}

impl<'a, 'b> Iterator for Iter<'a, 'b> where 'a: 'b {
    type Item = (&'a str, &'a Serialize);

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            None => {
                if let Some(parent) = self.properties.parent.clone() {
                    self.properties = parent;
                    self.iter = KeyValuesIter::over(self.properties.kvs);

                    self.iter.next()
                }
                else {
                    None
                }
            },
            item => item,
        }
    }
}

impl<'a> Properties<'a> {
    /// Iterate over the properties.
    pub fn iter<'b>(&'b self) -> Iter<'a, 'b> where 'a: 'b {
        self.into_iter()
    }

    /// Whether or not there are any properties.
    pub fn any(&self) -> bool {
        KeyValuesIter::over(self.kvs).any(|_| true) || self.parent.as_ref().map(|parent| parent.any()).unwrap_or(false)
    }
}

impl<'a, 'b> IntoIterator for &'b Properties<'a> where 'a: 'b {
    type IntoIter = Iter<'a, 'b>;
    type Item = (&'a str, &'a Serialize);

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            properties: &self,
            iter: KeyValuesIter::over(self.kvs)
        }
    }
}