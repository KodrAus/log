//! Structured key-value pairs for log records.

#[macro_use]
mod macros;

mod error;
mod key;

pub mod value;
pub mod source;

pub use self::error::Error;

#[doc(inline)]
pub use self::source::Source;
