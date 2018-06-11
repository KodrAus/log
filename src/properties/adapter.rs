pub mod map {
    use serde;
    use std::fmt::{Debug, Display};

    use properties::{Value, ToValue};
    use super::*;
    
    /// The default property adapter used when no `#[log]` attribute is present.
    /// 
    /// If `std` is available, this will use `Serialize`.
    /// If `std` is not available, this will use `Debug`.
    pub fn default(v: impl serde::Serialize + Debug) -> impl ToValue {
        #[cfg(feature = "erased-serde")]
        {
            serde(v)
        }
        #[cfg(not(feature = "erased-serde"))]
        {
            debug(v)
        }
    }

    /// `#[log(serde)]` Format a property value using its `Serialize` implementation.
    /// 
    /// The property value will retain its structure.
    #[cfg(feature = "erased-serde")]
    pub fn serde(v: impl serde::Serialize) -> impl ToValue {
        #[derive(Debug)]
        struct SerdeAdapter<T>(T);

        impl<T> ToValue for SerdeAdapter<T>
        where
            T: serde::Serialize,
        {
            fn to_value(&self) -> Value {
                Value::serde(&self.0)
            }
        }

        SerdeAdapter(v)
    }

    /// `#[log(debug)]` Format a property value using its `Debug` implementation.
    /// 
    /// The property value will be serialized as a string.
    pub fn debug(v: impl Debug) -> impl ToValue {
        map_with::fmt(v, Debug::fmt)
    }

    /// `#[log(display)]` Format a property value using its `Display` implementation.
    /// 
    /// The property value will be serialized as a string.
    pub fn display(v: impl Display) -> impl ToValue {
        map_with::fmt(v, Display::fmt)
    }
}

pub mod map_with {
    use std::fmt::{Debug, Display, Formatter, Result};

    use properties::{Value, ToValue};
    use super::*;

    /// `#[log(fmt = expr)]` Format a property value using a specific format.
    pub fn fmt<T>(value: T, adapter: impl Fn(&T, &mut Formatter) -> Result) -> impl ToValue {
        struct FmtAdapter<T, F> {
            value: T,
            adapter: F,
        }

        impl<T, F> Debug for FmtAdapter<T, F>
        where
            F: Fn(&T, &mut Formatter) -> Result,
        {
            fn fmt(&self, f: &mut Formatter) -> Result {
                (self.adapter)(&self.value, f)
            }
        }

        impl<T, F> ToValue for FmtAdapter<T, F>
        where
            F: Fn(&T, &mut Formatter) -> Result,
        {
            fn to_value(&self) -> Value {
                Value::fmt(self)
            }
        }

        FmtAdapter { value, adapter }
    }

    /// `#[log(with = expr)]` Use a generic adapter.
    pub fn with<T, U>(value: T, adapter: impl Fn(T) -> U) -> U
    where
        U: ToValue,
    {
        adapter(value)
    }
}