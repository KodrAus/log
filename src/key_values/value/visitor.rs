use std::fmt::Arguments;

use crate::key_values::value;

impl<'v> value::Value<'v> {
    /// Visit a value using a `Visitor`.
    pub fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), value::Error> {
        let mut backend = VisitorBackend(visitor);

        self.0.visit(&mut backend)
    }
}

/// A visitor for a value.
pub trait Visitor {
    /// Visit a format.
    fn fmt(&mut self, v: Arguments) -> Result<(), value::Error>;

    /// Visit an unsigned integer.
    fn u64(&mut self, v: u64) -> Result<(), value::Error> {
        self.fmt(format_args!("{:?}", v))
    }

    /// Visit a signed integer.
    fn i64(&mut self, v: i64) -> Result<(), value::Error> {
        self.fmt(format_args!("{:?}", v))
    }

    /// Visit a floating point number.
    fn f64(&mut self, v: f64) -> Result<(), value::Error> {
        self.fmt(format_args!("{:?}", v))
    }

    /// Visit a boolean.
    fn bool(&mut self, v: bool) -> Result<(), value::Error> {
        self.fmt(format_args!("{:?}", v))
    }

    /// Visit a Unicode character.
    fn char(&mut self, v: char) -> Result<(), value::Error> {
        self.fmt(format_args!("{:?}", v))
    }

    /// Visit a UTF-8 string.
    fn str(&mut self, v: &str) -> Result<(), value::Error> {
        self.fmt(format_args!("{:?}", v))
    }

    /// Visit an empty value.
    fn none(&mut self) -> Result<(), value::Error> {
        self.fmt(format_args!("{:?}", Option::None::<()>))
    }
}

impl<'a, T: ?Sized> Visitor for &'a mut T
where
    T: Visitor,
{
    fn fmt(&mut self, v: Arguments) -> Result<(), value::Error> {
        (**self).fmt(v)
    }

    fn u64(&mut self, v: u64) -> Result<(), value::Error> {
        (**self).u64(v)
    }

    fn i64(&mut self, v: i64) -> Result<(), value::Error> {
        (**self).i64(v)
    }

    fn f64(&mut self, v: f64) -> Result<(), value::Error> {
        (**self).f64(v)
    }

    fn bool(&mut self, v: bool) -> Result<(), value::Error> {
        (**self).bool(v)
    }

    fn char(&mut self, v: char) -> Result<(), value::Error> {
        (**self).char(v)
    }

    fn str(&mut self, v: &str) -> Result<(), value::Error> {
        (**self).str(v)
    }

    fn none(&mut self) -> Result<(), value::Error> {
        (**self).none()
    }
}

struct VisitorBackend<'a>(&'a mut dyn Visitor);

impl<'a> value::Backend for VisitorBackend<'a> {
    fn u64(&mut self, v: u64) -> Result<(), value::Error> {
        self.0.u64(v)
    }

    fn i64(&mut self, v: i64) -> Result<(), value::Error> {
        self.0.i64(v)
    }

    fn f64(&mut self, v: f64) -> Result<(), value::Error> {
        self.0.f64(v)
    }

    fn bool(&mut self, v: bool) -> Result<(), value::Error> {
        self.0.bool(v)
    }

    fn char(&mut self, v: char) -> Result<(), value::Error> {
        self.0.char(v)
    }

    fn str(&mut self, v: &str) -> Result<(), value::Error> {
        self.0.str(v)
    }

    fn none(&mut self) -> Result<(), value::Error> {
        self.0.none()
    }
}

impl<'a> value::fmt::Backend for VisitorBackend<'a> {
    fn debug(&mut self, v: &dyn value::fmt::Value) -> Result<(), value::Error> {
        self.0.fmt(format_args!("{:?}", v))
    }
}

#[cfg(feature = "kv_sval")]
impl<'a> value::sval::Backend for VisitorBackend<'a> {
    fn sval(&mut self, v: &dyn value::sval::Value) -> Result<(), value::Error> {
        self.0.fmt(format_args!("{:?}", v))
    }
}

#[cfg(feature = "kv_serde")]
impl<'a> serde::Backend for VisitorBackend<'a> {
    fn serde(&mut self, v: &dyn value::serde::Value) -> Result<(), value::Error> {
        self.0.fmt(format_args!("{:?}", v))
    }
}
