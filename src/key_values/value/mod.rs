//! Structured values.

use std::mem;
use std::fmt::Arguments;
use std::marker::PhantomData;

mod impls;
mod fmt;
mod sval;
mod serde;

#[doc(inline)]
pub use super::Error;

/// A type that can be converted into a value.
pub trait ToValue {
    /// Perform the conversion.
    fn to_value(&self) -> Value;
}

/// The value in a key-value pair.
pub struct Value<'v>(Inner<'v>);

impl<'v> Value<'v> {
    /// Create a value from some type.
    /// 
    /// The value must be provided with a compatible from method,
    /// but doesn't need to implement any traits. This method is
    /// useful when the type `T` can't satisfy the requirements
    /// for other `Value::from` methods, but the lifetime `'v`
    /// prevents local new-types from being used.
    pub fn from_any<T>(v: &'v T, from: FromAnyFn<T>) -> Self {
        Value(Inner::new(v, from))
    }

    /// Visit a value using a `Visitor`.
    pub fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
        let mut backend = VisitorBackend(visitor);

        self.0.visit(&mut backend)
    }
}

impl<'v> ToValue for Value<'v> {
    fn to_value(&self) -> Value {
        Value(self.0)
    }
}

// Pinched from `libstd::fmt`

struct Void {
    _priv: (),
    /// Erases all oibits, because `Void` erases the type of the object that
    /// will be used to produce formatted output. Since we do not know what
    /// oibits the real types have (and they can have any or none), we need to
    /// take the most conservative approach and forbid all oibits.
    ///
    /// It was added after #45197 showed that one could share a `!Sync`
    /// object across threads by passing it into `format_args!`.
    _oibit_remover: PhantomData<*mut dyn Fn()>,
}

#[derive(Clone, Copy)]
struct Inner<'a> {
    data: &'a Void,
    from: FromAnyFn<Void>,
}

type FromAnyFn<T> = fn(FromAny, &T) -> Result<(), Error>;

impl<'a> Inner<'a> {
    fn new<T>(data: &'a T, from: FromAnyFn<T>) -> Self {
        unsafe {
            Inner {
                data: mem::transmute::<&'a T, &'a Void>(data),
                from: mem::transmute::<FromAnyFn<T>, FromAnyFn<Void>>(from),
            }
        }
    }

    fn visit(&self, backend: &mut dyn Backend) -> Result<(), Error> {
        (self.from)(FromAny(backend), self.data)
    }
}

/// A visitor for a value.
pub trait Visitor {
    /// Visit a format.
    fn fmt(&mut self, v: Arguments) -> Result<(), Error>;

    /// Visit an unsigned integer.
    fn u64(&mut self, v: u64) -> Result<(), Error> {
        self.fmt(format_args!("{:?}", v))
    }

    /// Visit a signed integer.
    fn i64(&mut self, v: i64) -> Result<(), Error> {
        self.fmt(format_args!("{:?}", v))
    }

    /// Visit a floating point number.
    fn f64(&mut self, v: f64) -> Result<(), Error> {
        self.fmt(format_args!("{:?}", v))
    }

    /// Visit a boolean.
    fn bool(&mut self, v: bool) -> Result<(), Error> {
        self.fmt(format_args!("{:?}", v))
    }

    /// Visit a Unicode character.
    fn char(&mut self, v: char) -> Result<(), Error> {
        self.fmt(format_args!("{:?}", v))
    }

    /// Visit a UTF-8 string.
    fn str(&mut self, v: &str) -> Result<(), Error> {
        self.fmt(format_args!("{:?}", v))
    }

    /// Visit an empty value.
    fn none(&mut self) -> Result<(), Error> {
        self.fmt(format_args!("{:?}", Option::None::<()>))
    }
}

impl<'a, T: ?Sized> Visitor for &'a mut T
where
    T: Visitor,
{
    fn fmt(&mut self, v: Arguments) -> Result<(), Error> {
        (**self).fmt(v)
    }

    fn u64(&mut self, v: u64) -> Result<(), Error> {
        (**self).u64(v)
    }

    fn i64(&mut self, v: i64) -> Result<(), Error> {
        (**self).i64(v)
    }

    fn f64(&mut self, v: f64) -> Result<(), Error> {
        (**self).f64(v)
    }

    fn bool(&mut self, v: bool) -> Result<(), Error> {
        (**self).bool(v)
    }

    fn char(&mut self, v: char) -> Result<(), Error> {
        (**self).char(v)
    }

    fn str(&mut self, v: &str) -> Result<(), Error> {
        (**self).str(v)
    }

    fn none(&mut self) -> Result<(), Error> {
        (**self).none()
    }
}

struct VisitorBackend<'a>(&'a mut dyn Visitor);

impl<'a> Backend for VisitorBackend<'a> {
    fn u64(&mut self, v: u64) -> Result<(), Error> {
        self.0.u64(v)
    }

    fn i64(&mut self, v: i64) -> Result<(), Error> {
        self.0.i64(v)
    }

    fn f64(&mut self, v: f64) -> Result<(), Error> {
        self.0.f64(v)
    }

    fn bool(&mut self, v: bool) -> Result<(), Error> {
        self.0.bool(v)
    }

    fn char(&mut self, v: char) -> Result<(), Error> {
        self.0.char(v)
    }

    fn str(&mut self, v: &str) -> Result<(), Error> {
        self.0.str(v)
    }

    fn none(&mut self) -> Result<(), Error> {
        self.0.none()
    }
}

impl<'a> fmt::Backend for VisitorBackend<'a> {
    fn debug(&mut self, v: &dyn fmt::Value) -> Result<(), Error> {
        self.0.fmt(format_args!("{:?}", v))
    }
}

#[cfg(feature = "kv_sval")]
impl<'a> sval::Backend for VisitorBackend<'a> {
    fn sval(&mut self, v: &dyn sval::Value) -> Result<(), Error> {
        self.0.fmt(format_args!("{:?}", v))
    }
}

#[cfg(feature = "kv_serde")]
impl<'a> serde::Backend for VisitorBackend<'a> {
    fn serde(&mut self, v: &dyn serde::Value) -> Result<(), Error> {
        self.0.fmt(format_args!("{:?}", v))
    }
}

/// A builder for a value.
/// 
/// An instance of this type is passed to the `Value::from_any` method.
pub struct FromAny<'a>(&'a mut dyn Backend);

impl<'a> FromAny<'a> {
    fn value(self, v: Value) -> Result<(), Error> {
        v.0.visit(self.0)
    }

    fn u64(self, v: u64) -> Result<(), Error> {
        self.0.u64(v)
    }

    fn i64(self, v: i64) -> Result<(), Error> {
        self.0.i64(v)
    }
    
    fn f64(self, v: f64) -> Result<(), Error> {
        self.0.f64(v)
    }

    fn bool(self, v: bool) -> Result<(), Error> {
        self.0.bool(v)
    }

    fn char(self, v: char) -> Result<(), Error> {
        self.0.char(v)
    }

    fn none(self) -> Result<(), Error> {
        self.0.none()
    }

    fn str(self, v: &str) -> Result<(), Error> {
        self.0.str(v)
    }
}

/// A backend that can receive the structure of a `Value`.
/// 
/// In addition to the primitives defined here each backend must also support
/// values from any other backend.
trait Backend: self::fmt::Backend + self::sval::Backend + self::serde::Backend {
    fn u64(&mut self, v: u64) -> Result<(), Error>;
    fn i64(&mut self, v: i64) -> Result<(), Error>;
    fn f64(&mut self, v: f64) -> Result<(), Error>;
    fn bool(&mut self, v: bool) -> Result<(), Error>;
    fn char(&mut self, v: char) -> Result<(), Error>;
    fn str(&mut self, v: &str) -> Result<(), Error>;
    fn none(&mut self) -> Result<(), Error>;
}
