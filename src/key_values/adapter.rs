//! Adapters for key values in the `log` macros.

pub mod map {
    //! Receivers for `#[log($adapter)]` attributes.
    
    use std::fmt::{Debug, Display};
    #[cfg(any(feature = "alloc", feature = "std"))]
    use std::path::Path;

    #[cfg(feature = "erased-serde")]
    use serde;
    
    #[cfg(feature = "erased-serde")]
    use key_values::Value;
    use key_values::ToValue;
    use super::*;
    
    /// The default property adapter used when no `#[log]` attribute is present.
    pub fn default(v: impl ToValue) -> impl ToValue {
        v
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
                Value::from_serde(&self.0)
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

    /// `#[log(path)]` Format a property value as a path.
    /// 
    /// If the path contains invalid UTF8 characters then they will be escaped.
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub fn path(v: impl AsRef<Path>) -> impl ToValue {
        use std::fmt;
        
        #[derive(Debug)]
        struct PathAdapter<T>(T);

        impl<T> Display for PathAdapter<T>
        where
            T: AsRef<Path>,
        {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                let path = self.0.as_ref();

                match path.to_str() {
                    Some(path) => Display::fmt(path, f),
                    None => Debug::fmt(path, f),
                }
            }
        }

        impl<T> serde::Serialize for PathAdapter<T>
        where
            T: AsRef<Path>,
        {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer
            {
                serializer.collect_str(&self)
            }
        }

        PathAdapter(v)
    }
}

pub mod map_with {
    //! Receivers for `#[log($adapter = $state)]` attributes.
    
    use std::fmt::{Display, Formatter, Result};

    use key_values::{Value, ToValue};

    /// `#[log(fmt = expr)]` Format a property value using a specific format.
    pub fn fmt<T>(value: T, adapter: impl Fn(&T, &mut Formatter) -> Result) -> impl ToValue {
        #[derive(Debug)]
        struct FmtAdapter<T, F> {
            value: T,
            adapter: F,
        }

        impl<T, F> Display for FmtAdapter<T, F>
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
                Value::from_display(self)
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