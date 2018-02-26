//! Log record properties.

use std::{fmt, error};
use std::collections::BTreeMap;
use std::borrow::Borrow;
use std::collections::Bound;
use std::marker::PhantomData;

use serde::ser::{self, Serialize as SerdeSerialize};

pub use erased_serde::Serialize;

/// A single property with a key and a value.
pub struct Property<'a> {
    key: &'a str,
    value: &'a Serialize,
}

impl<'a> Property<'a> {
    /// The key associated with this property.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// The value associated with this property.
    pub fn value(&self) -> &Serialize {
        &self.value
    }

    fn with<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(Value) -> Option<R>
    {
        self.value.serialize(ValueSerializer::new(f))
        .ok()
        .and_then(|r| r)
    }

    /// Use the value as a boolean.
    /// 
    /// If the property value is a boolean, the closure will be executed.
    /// If the property value is not a boolean, then `None` is returned.
    pub fn with_bool<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(bool) -> R
    {
        self.with(|v| match v {
            Value::Bool(b) => Some(f(b)),
            _ => None
        })
    }

    /// Use the value as a signed integer.
    /// 
    /// If the property value is a signed integer, the closure will be executed.
    /// If the property value is not a signed integer, then `None` is returned.
    pub fn with_i64<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(i64) -> R
    {
        self.with(|v| match v {
            Value::I64(n) => Some(f(n)),
            _ => None
        })
    }

    /// Use the value as an unsigned integer.
    /// 
    /// If the property value is an unsigned integer, the closure will be executed.
    /// If the property value is not an unsigned integer, then `None` is returned.
    pub fn with_u64<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(u64) -> R
    {
        self.with(|v| match v {
            Value::U64(n) => Some(f(n)),
            _ => None
        })
    }

    /// Use the value as a borrowed string.
    /// 
    /// This makes it possible to inspect a possible string property without
    /// having to copy it into an owned `String` first.
    /// If the property value is a string the closure will be executed.
    /// If the property value is not a string then `None` is returned.
    pub fn with_str<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&str) -> R
    {
        self.with(|v| match v {
            Value::String(s) => Some(f(s)),
            _ => None
        })
    }

    /// Get the value as a boolean.
    pub fn as_bool(&self) -> Option<bool> {
        self.with_bool(|b| b)
    }

    /// Get the value as a signed integer.
    pub fn as_i8(&self) -> Option<i8> {
        self.with_i64(|n| n as i8)
    }

    /// Get the value as a signed integer.
    pub fn as_i16(&self) -> Option<i16> {
        self.with_i64(|n| n as i16)
    }

    /// Get the value as a signed integer.
    pub fn as_i32(&self) -> Option<i32> {
        self.with_i64(|n| n as i32)
    }

    /// Get the value as a signed integer.
    pub fn as_i64(&self) -> Option<i64> {
        self.with_i64(|n| n as i64)
    }

    /// Get the value as an unsigned integer.
    pub fn as_u8(&self) -> Option<u8> {
        self.with_i64(|n| n as u8)
    }

    /// Get the value as an unsigned integer.
    pub fn as_u16(&self) -> Option<u16> {
        self.with_i64(|n| n as u16)
    }

    /// Get the value as an unsigned integer.
    pub fn as_u32(&self) -> Option<u32> {
        self.with_i64(|n| n as u32)
    }

    /// Get the value as an unsigned integer.
    pub fn as_u64(&self) -> Option<u64> {
        self.with_i64(|n| n as u64)
    }

    /// Get the value as an owned string.
    pub fn as_string(&self) -> Option<String> {
        self.with_str(|s| s.to_owned())
    }
}

impl<'a> fmt::Debug for Property<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Entry").finish()
    }
}

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
    pub fn as_u64(&self) -> Option<u64> {
        match self.0 {
            KeyInner::Number(n) => Some(n),
            _ => None
        }
    }

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

/// A raw set of key value pairs.
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

/// A chain of properties.
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

        for property in self {
            d.entry(&property.key());
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

#[derive(Clone, Copy)]
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
    type Item = Property<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(Entry { key, value, next, .. }) = self.current.take() {
            let next = next.and_then(|ref key| self.kvs.entry(key));
            self.current = next;

            Some(Property {
                key,
                value,
            })
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
    type Item = Property<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            None => {
                if let Some(parent) = self.properties.parent {
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
    type Item = Property<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            properties: &self,
            iter: KeyValuesIter::over(self.kvs)
        }
    }
}

struct ValueSerializer<F, R> {
    f: F,
    _marker: PhantomData<Fn() -> R>
}

impl<F, R> ValueSerializer<F, R>
where
    F: FnOnce(Value) -> R
{
    fn new(f: F) -> Self {
        ValueSerializer {
            f,
            _marker: PhantomData
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct UnsupportedValue;

impl error::Error for UnsupportedValue {
    fn cause(&self) -> Option<&error::Error> {
        None
    }

    fn description(&self) -> &str {
        "not a supported value"
    }
}

impl fmt::Display for UnsupportedValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "not a supported value")
    }
}

impl ser::Error for UnsupportedValue {
    fn custom<T>(_: T) -> Self
    where
        T: fmt::Display
    {
        UnsupportedValue
    }
}

enum Value<'a> {
    Bool(bool),
    String(&'a str),
    I64(i64),
    U64(u64),
    F64(f64),
}

impl<F, R> ser::Serializer for ValueSerializer<F, R>
where
    F: FnOnce(Value) -> R
{
    type Ok = R;
    type Error = UnsupportedValue;

    type SerializeSeq = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeMap = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = ser::Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = ser::Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok((self.f)(Value::Bool(v)))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok((self.f)(Value::I64(v)))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok((self.f)(Value::U64(v)))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as f64)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok((self.f)(Value::F64(v)))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok((self.f)(Value::String(v)))
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
        where T: ?Sized + ser::Serialize
    {
        value.serialize(self)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Self::Ok, Self::Error>
        where T: ?Sized + ser::Serialize
    {
        value.serialize(self)
    }

    fn serialize_char(self, _: char) -> Result<Self::Ok, Self::Error> {
        Err(UnsupportedValue)
    }

    fn serialize_bytes(self, _: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(UnsupportedValue)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(UnsupportedValue)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(UnsupportedValue)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str
    ) -> Result<Self::Ok, Self::Error> {
        Err(UnsupportedValue)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T
    ) -> Result<Self::Ok, Self::Error>
        where T: ?Sized + Serialize
    {
        Err(UnsupportedValue)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(UnsupportedValue)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(UnsupportedValue)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(UnsupportedValue)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(UnsupportedValue)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(UnsupportedValue)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(UnsupportedValue)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(UnsupportedValue)
    }
}