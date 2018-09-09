//! Log property values.
//! 
//! This module contains the `Value` type,
//! which is roughly a `dyn Serialize + Display`.

use std::fmt;
use serde;

#[cfg(feature = "erased-serde")]
use erased_serde;

#[cfg(not(feature = "erased-serde"))]
use super::primitive::ToPrimitive;

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
#[derive(Clone)]
pub struct Value<'a> {
    inner: ValueInner<'a>,
}

#[derive(Clone, Copy)]
enum ValueInner<'a> {
    Display(&'a dyn fmt::Display),
    #[cfg(feature = "erased-serde")]
    Serde(&'a dyn erased_serde::Serialize),
    #[cfg(not(feature = "erased-serde"))]
    Primitive(&'a dyn ToPrimitive),
}

impl<'a> ToValue for Value<'a> {
    fn to_value(&self) -> Value {
        Value { inner: self.inner }
    }
}

impl<'a> Value<'a> {
    /// Create a new value.
    /// 
    /// The value must implement both `serde::Serialize` and `fmt::Display`.
    /// Either implementation will be used depending on whether the standard
    /// library is available, but is exposed through the same API.
    /// 
    /// In environments where the standard library is available, the `Serialize`
    /// implementation will be used.
    /// 
    /// In environments where the standard library is not available, some
    /// primitive stack-based values can retain their structure instead of falling
    /// back to `Display`.
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
                    if v.is_primitive() {
                        ValueInner::Primitive(v)
                    } else {
                        ValueInner::Display(v)
                    }
                }
            }
        }
    }

    /// Get a `Value` from a displayable reference.
    pub fn from_display(v: &'a impl fmt::Display) -> Self {
        Value {
            inner: ValueInner::Display(v),
        }
    }

    /// Get a `Value` from a serializable reference.
    #[cfg(feature = "erased-serde")]
    pub fn from_serde(v: &'a impl serde::Serialize) -> Self {
        Value {
            inner: ValueInner::Serde(v),
        }
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
            ValueInner::Primitive(v) => {
                use serde::ser::Error as SerError;

                // We expect `Value::new` to correctly determine
                // whether or not a value is a simple primitive
                let v = v
                    .to_primitive()
                    .ok_or_else(|| S::Error::custom("captured value is not primitive"))?;

                v.serialize(serializer)
            },
        }
    }
}

impl<'a> fmt::Debug for Value<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Value").finish()
    }
}
