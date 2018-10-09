use std::fmt;

use serde;

#[cfg(feature = "std")]
use std::error;

pub struct Error(Inner);

impl Error {
    pub fn msg(msg: &'static str) -> Self {
        Error(Inner::Static(msg))
    }

    #[cfg(feature = "std")]
    pub fn as_error(&self) -> &(dyn error::Error + Send + Sync + 'static) {
        &self.0
    }

    #[cfg(feature = "std")]
    pub fn into_error(self) -> Box<dyn error::Error + Send + Sync> {
        Box::new(self.0)
    }

    pub fn into_serde<E>(self) -> E
    where
        E: serde::ser::Error,
    {
        E::custom(self)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "std")]
impl<E> From<E> for Error
where
    E: error::Error,
{
    fn from(err: E) -> Self {
        Error(Inner::Owned(err.to_string()))
    }
}

#[cfg(feature = "std")]
impl From<Error> for Box<dyn error::Error + Send + Sync> {
    fn from(err: Error) -> Self {
        err.into_error()
    }
}

impl AsRef<dyn error::Error + Send + Sync + 'static> for Error {
    fn as_ref(&self) -> &(dyn error::Error + Send + Sync + 'static) {
        self.as_error()
    }
}

enum Inner {
    Static(&'static str),
    #[cfg(feature = "std")]
    Owned(String),
}

impl fmt::Debug for Inner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Inner::Static(msg) => msg.fmt(f),
            #[cfg(feature = "std")]
            Inner::Owned(msg) => msg.fmt(f),
        }
    }
}

impl fmt::Display for Inner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Inner::Static(msg) => msg.fmt(f),
            #[cfg(feature = "std")]
            Inner::Owned(msg) => msg.fmt(f),
        }
    }
}

#[cfg(feature = "std")]
impl error::Error for Inner {
    fn description(&self) -> &str {
        match self {
            Inner::Static(msg) => msg,
            Inner::Owned(msg) => msg,
        }
    }
}
