//! Serialization for structured values.

use std::mem;
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
pub struct Value<'v>(Any<'v>);

impl<'v> Value<'v> {
    /// Create a value from an anonymous type.
    /// 
    /// The value must be provided with a compatible visit method.
    pub fn from_any<T>(v: &'v T, visit: VisitFn<T>) -> Self {
        Value(Any::new(v, visit))
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
struct Any<'a> {
    data: &'a Void,
    visit: VisitFn<Void>,
}

type VisitFn<T> = fn(Visitor, &T) -> Result<(), Error>;

impl<'a> Any<'a> {
    fn new<T>(data: &'a T, visit: VisitFn<T>) -> Self {
        unsafe {
            Any {
                data: mem::transmute::<&'a T, &'a Void>(data),
                visit: mem::transmute::<VisitFn<T>, VisitFn<Void>>(visit),
            }
        }
    }

    fn visit(&self, visitor: Visitor) -> Result<(), Error> {
        (self.visit)(visitor, self.data)
    }
}

/// A visitor for a value.
pub struct Visitor<'a>(&'a mut dyn Backend);

impl<'a> Visitor<'a> {
    fn u64(self, v: u64) -> Result<(), Error> {
        self.0.u64(v)
    }

    fn i64(self, v: i64) -> Result<(), Error> {
        self.0.i64(v)
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

    fn str(&mut self, v: &str) -> Result<(), Error>;
}
