use std::fmt;
use serde;

#[cfg(feature = "erased-serde")]
use erased_serde;

use super::primitive::Primitive;

/// Converting into a `Value`.
pub trait ToValue {
    /// Perform the conversion.
    fn to_value(&self) -> Value;
}

impl<T: serde::Serialize + fmt::Display> ToValue for T {
    fn to_value(&self) -> Value {
        Value::new(self)
    }
}

impl<'a> ToValue for &'a dyn ToValue {
    fn to_value(&self) -> Value {
        (*self).to_value()
    }
}

/// A single property value.
/// 
/// Values borrow their underlying data and implement `serde::Serialize`.
#[derive(Clone, Copy)]
#[allow(missing_debug_implementations)]
pub struct Value<'a> {
    inner: ValueInner<'a>,
}

#[derive(Clone, Copy)]
enum ValueInner<'a> {
    Display(&'a dyn fmt::Display),
    #[cfg(feature = "erased-serde")]
    Serde(&'a dyn erased_serde::Serialize),
    #[cfg(not(feature = "erased-serde"))]
    Primitive(Primitive),
}

impl<'a> ToValue for Value<'a> {
    fn to_value(&self) -> Value {
        Value { inner: self.inner }
    }
}

impl<'a> serde::Serialize for Value<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.inner {
            ValueInner::Display(v) => serializer.collect_str(&v),

            #[cfg(feature = "erased-serde")]
            ValueInner::Serde(v) => v.serialize(serializer),

            #[cfg(not(feature = "erased-serde"))]
            ValueInner::Primitive(v) => v.serialize(serializer),
        }
    }
}

impl<'a> Value<'a> {
    /// Create a new value.
    /// 
    /// The value must implement both `serde::Serialize` and `fmt::Display`.
    /// Either implementation will be used depending on whether the standard
    /// library is available, but is exposed through the same API.
    pub fn new(v: &'a (impl serde::Serialize + fmt::Display)) -> Self {
        Value {
            inner: {
                #[cfg(feature = "erased-serde")]
                {
                    ValueInner::Serde(v)
                }

                #[cfg(not(feature = "erased-serde"))]
                {
                    // Try capture a primitive value
                    // If we can represent it on the stack then we can avoid using
                    // the `Display` implementation
                    if let Some(primitive) = Primitive::try_from(v) {
                        ValueInner::Primitive(primitive)
                    } else {
                        ValueInner::Display(v)
                    }
                }
            }
        }
    }

    /// Get a `Value` from some displayable reference.
    pub fn from_display(v: &'a impl fmt::Display) -> Self {
        Value {
            inner: ValueInner::Display(v),
        }
    }

    /// Get a `Value` from some serializable reference.
    #[cfg(feature = "erased-serde")]
    pub fn from_serde(v: &'a impl serde::Serialize) -> Self {
        Value {
            inner: ValueInner::Serde(v),
        }
    }
}
