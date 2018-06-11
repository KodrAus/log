use std::fmt;

use serde;

#[cfg(feature = "erased-serde")]
use erased_serde;

use properties;

/// A single property value.
/// 
/// Values implement `serde::Serialize`.
pub struct Value<'a> {
    inner: ValueInner<'a>,
}

#[derive(Clone, Copy)]
enum ValueInner<'a> {
    Fmt(&'a dyn fmt::Debug),
    #[cfg(feature = "erased-serde")]
    Serde(&'a dyn erased_serde::Serialize),
}

impl<'a> serde::Serialize for Value<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.inner {
            ValueInner::Fmt(v) => {
                struct FmtAdapter<T>(T);

                impl<T> fmt::Display for FmtAdapter<T>
                where
                    T: fmt::Debug,
                {
                    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        self.0.fmt(f)
                    }
                }

                serializer.collect_str(&FmtAdapter(v))
            },
            #[cfg(feature = "erased-serde")]
            ValueInner::Serde(v) => v.serialize(serializer),
        }
    }
}

impl<'a> Value<'a> {
    pub fn new(v: &'a (impl serde::Serialize + fmt::Debug)) -> Self {
        Value {
            inner: {
                #[cfg(feature = "erased-serde")]
                {
                    ValueInner::Serde(v)
                }
                #[cfg(not(feature = "erased-serde"))]
                {
                    ValueInner::Fmt(v)
                }
            }
        }
    }

    pub fn fmt(v: &'a impl fmt::Debug) -> Self {
        Value {
            inner: ValueInner::Fmt(v),
        }
    }

    #[cfg(feature = "erased-serde")]
    pub fn serde(v: &'a impl serde::Serialize) -> Self {
        Value {
            inner: ValueInner::Serde(v),
        }
    }
}

impl<'a> fmt::Debug for Value<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Value").finish()
    }
}

pub trait ToValue {
    fn to_value(&self) -> Value;
}

impl<'a> ToValue for Value<'a> {
    fn to_value(&self) -> Value {
        Value { inner: self.inner }
    }
}
