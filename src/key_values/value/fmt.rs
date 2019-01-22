/*
A `std::fmt` backend for structured values.

This module allows capturing `impl std::fmt::Debug` as a `Value`,
and formatting any `Value` using `std::fmt`.
*/

use std::fmt;

use crate::key_values::value;

impl<'v> value::Value<'v> {
    /// Create a value from a `fmt::Debug`.
    pub fn from_debug(v: &'v impl fmt::Debug) -> Self {
        Self::from_any(v, |from, v| from.debug(v))
    }
}

impl<'v> fmt::Debug for value::Value<'v> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.visit(&mut FmtBackend(f)).map_err(|_| fmt::Error)
    }
}

impl<'v> fmt::Display for value::Value<'v> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<'a> value::FromAny<'a> {
    /// Visit a value that can be formatted.
    pub fn debug(self, v: impl fmt::Debug) -> Result<(), value::Error> {
        self.0.debug(&v)
    }
}

pub(in crate::key_values::value) trait Backend {
    fn debug(&mut self, v: &dyn Value) -> Result<(), value::Error>;
}

pub(in crate::key_values::value) use fmt::Debug as Value;

struct FmtBackend<'a, 'b>(&'a mut fmt::Formatter<'b>);

impl<'a, 'b> value::Backend for FmtBackend<'a, 'b> {
    fn u64(&mut self, v: u64) -> Result<(), value::Error> {
        self.debug(&v)
    }

    fn i64(&mut self, v: i64) -> Result<(), value::Error> {
        self.debug(&v)
    }

    fn f64(&mut self, v: f64) -> Result<(), value::Error> {
        self.debug(&v)
    }

    fn bool(&mut self, v: bool) -> Result<(), value::Error> {
        self.debug(&v)
    }

    fn char(&mut self, v: char) -> Result<(), value::Error> {
        self.debug(&v)
    }

    fn none(&mut self) -> Result<(), value::Error> {
        self.debug(&Option::None::<()>)
    }

    fn str(&mut self, v: &str) -> Result<(), value::Error> {
        self.debug(&v)
    }
}

impl<'a, 'b> Backend for FmtBackend<'a, 'b> {
    fn debug(&mut self, v: &dyn fmt::Debug) -> Result<(), value::Error> {
        write!(self.0, "{:?}", v)?;

        Ok(())
    }
}

#[cfg(feature = "kv_sval")]
impl<'a, 'b> value::sval::Backend for FmtBackend<'a, 'b> {
    fn sval(&mut self, v: &dyn value::sval::Value) -> Result<(), value::Error> {
        self.debug(&v)
    }
}

#[cfg(feature = "kv_serde")]
impl<'a, 'b> value::serde::Backend for FmtBackend<'a, 'b> {
    fn serde(&mut self, v: &dyn value::serde::Value) -> Result<(), value::Error> {
        self.debug(&v)
    }
}
