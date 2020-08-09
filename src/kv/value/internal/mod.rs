//! The internal `Value` serialization API.
//!
//! This implementation isn't intended to be public. It may need to change
//! for optimizations or to support new external serialization frameworks.

use std::any::TypeId;

use super::{Error, Fill, Slot};

pub(super) mod cast;
#[cfg(feature = "std")]
pub(super) mod error;
pub(super) mod fmt;
#[cfg(feature = "kv_unstable_sval")]
pub(super) mod sval;

// NOTE: Right now this `Inner` type is *huge* (~64 bytes)
// It's written to be straightforward, but could be optimized to get its size down

/// A container for a structured value for a specific kind of visitor.
#[derive(Clone, Copy)]
pub(super) enum Inner<'v> {
    /// A simple primitive value that can be copied without allocating.
    Primitive { value: Primitive<'v> },
    /// A value that can be filled.
    Fill { value: &'v dyn Fill },
    /// A debuggable value.
    Debug {
        value: &'v dyn fmt::Debug,
        type_id: Option<TypeId>,
    },
    /// A displayable value.
    Display {
        value: &'v dyn fmt::Display,
        type_id: Option<TypeId>,
    },

    #[cfg(feature = "std")]
    /// An error.
    Error {
        value: &'v dyn error::Error,
        type_id: Option<TypeId>,
    },

    #[cfg(feature = "kv_unstable_sval")]
    /// A structured value from `sval`.
    Sval {
        value: &'v dyn sval::Value,
        type_id: Option<TypeId>,
    },
}

impl<'v> Inner<'v> {
    pub(super) fn visit(self, visitor: &mut dyn Visitor<'v>) -> Result<(), Error> {
        match self {
            Inner::Primitive { value } => value.visit(visitor),
            Inner::Fill { value } => value.fill(&mut Slot::new(visitor)),

            Inner::Debug { value, .. } => visitor.debug(value),
            Inner::Display { value, .. } => visitor.display(value),

            #[cfg(feature = "std")]
            Inner::Error { value, .. } => visitor.error(value),

            #[cfg(feature = "kv_unstable_sval")]
            Inner::Sval { value, .. } => visitor.sval(value),
        }
    }
}

/// The internal serialization contract.
pub(super) trait Visitor<'v> {
    fn debug(&mut self, v: &dyn fmt::Debug) -> Result<(), Error>;
    fn display(&mut self, v: &dyn fmt::Display) -> Result<(), Error> {
        self.debug(&format_args!("{}", v))
    }

    fn u64(&mut self, v: u64) -> Result<(), Error>;
    fn i64(&mut self, v: i64) -> Result<(), Error>;
    fn f64(&mut self, v: f64) -> Result<(), Error>;
    fn bool(&mut self, v: bool) -> Result<(), Error>;
    fn char(&mut self, v: char) -> Result<(), Error>;

    fn str(&mut self, v: &str) -> Result<(), Error>;
    fn borrowed_str(&mut self, v: &'v str) -> Result<(), Error> {
        self.str(v)
    }

    fn none(&mut self) -> Result<(), Error>;

    #[cfg(feature = "std")]
    fn error(&mut self, v: &dyn error::Error) -> Result<(), Error>;

    #[cfg(feature = "kv_unstable_sval")]
    fn sval(&mut self, v: &dyn sval::Value) -> Result<(), Error>;
}

/// A captured primitive value.
///
/// These values are common and cheap to copy around.
#[derive(Clone, Copy)]
pub(super) enum Primitive<'v> {
    Signed(i64),
    Unsigned(u64),
    Float(f64),
    Bool(bool),
    Char(char),
    Str(&'v str),
    None,
}

impl<'v> Primitive<'v> {
    fn visit(self, visitor: &mut dyn Visitor<'v>) -> Result<(), Error> {
        match self {
            Primitive::Signed(value) => visitor.i64(value),
            Primitive::Unsigned(value) => visitor.u64(value),
            Primitive::Float(value) => visitor.f64(value),
            Primitive::Bool(value) => visitor.bool(value),
            Primitive::Char(value) => visitor.char(value),
            Primitive::Str(value) => visitor.borrowed_str(value),
            Primitive::None => visitor.none(),
        }
    }
}

impl<'v> From<u8> for Primitive<'v> {
    #[inline]
    fn from(v: u8) -> Self {
        Primitive::Unsigned(v as u64)
    }
}

impl<'v> From<u16> for Primitive<'v> {
    #[inline]
    fn from(v: u16) -> Self {
        Primitive::Unsigned(v as u64)
    }
}

impl<'v> From<u32> for Primitive<'v> {
    #[inline]
    fn from(v: u32) -> Self {
        Primitive::Unsigned(v as u64)
    }
}

impl<'v> From<u64> for Primitive<'v> {
    #[inline]
    fn from(v: u64) -> Self {
        Primitive::Unsigned(v)
    }
}

impl<'v> From<usize> for Primitive<'v> {
    #[inline]
    fn from(v: usize) -> Self {
        Primitive::Unsigned(v as u64)
    }
}

impl<'v> From<i8> for Primitive<'v> {
    #[inline]
    fn from(v: i8) -> Self {
        Primitive::Signed(v as i64)
    }
}

impl<'v> From<i16> for Primitive<'v> {
    #[inline]
    fn from(v: i16) -> Self {
        Primitive::Signed(v as i64)
    }
}

impl<'v> From<i32> for Primitive<'v> {
    #[inline]
    fn from(v: i32) -> Self {
        Primitive::Signed(v as i64)
    }
}

impl<'v> From<i64> for Primitive<'v> {
    #[inline]
    fn from(v: i64) -> Self {
        Primitive::Signed(v)
    }
}

impl<'v> From<isize> for Primitive<'v> {
    #[inline]
    fn from(v: isize) -> Self {
        Primitive::Signed(v as i64)
    }
}

impl<'v> From<f32> for Primitive<'v> {
    #[inline]
    fn from(v: f32) -> Self {
        Primitive::Float(v as f64)
    }
}

impl<'v> From<f64> for Primitive<'v> {
    #[inline]
    fn from(v: f64) -> Self {
        Primitive::Float(v)
    }
}

impl<'v> From<bool> for Primitive<'v> {
    #[inline]
    fn from(v: bool) -> Self {
        Primitive::Bool(v)
    }
}

impl<'v> From<char> for Primitive<'v> {
    #[inline]
    fn from(v: char) -> Self {
        Primitive::Char(v)
    }
}

impl<'v> From<&'v str> for Primitive<'v> {
    #[inline]
    fn from(v: &'v str) -> Self {
        Primitive::Str(v)
    }
}
