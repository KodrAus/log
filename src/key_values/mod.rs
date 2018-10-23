//! Structured key-value pairs for log records.

#[macro_use]
mod macros;

mod error;
mod value;
mod key;
pub mod source;

pub use self::error::Error;
pub use self::key::Key;
pub use self::value::{Value, ValueVisitor};

#[doc(inline)]
pub use self::source::{Source, SourceVisitor};
