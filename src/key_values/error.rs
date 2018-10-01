use std::fmt;

#[cfg(feature = "std")]
use std::error;

pub struct Error(Inner);

impl Error {
    pub fn msg(msg: &'static str) -> Self {
        Error(Inner::Static(msg))
    }

    #[cfg(feature = "std")]
    pub fn boxed(err: impl error::Error + Send + Sync + 'static) -> Self {
        Error(Inner::Boxed(Box::new(err)))
    }

    #[cfg(feature = "std")]
    fn as_error(&self) -> &(dyn error::Error + Send + Sync + 'static) {
        &self.0
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

enum Inner {
    Static(&'static str),
    #[cfg(feature = "std")]
    Owned(String),
    #[cfg(feature = "std")]
    Boxed(Box<dyn error::Error + Send + Sync>),
}

impl fmt::Debug for Inner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Inner::Static(msg) => msg.fmt(f),
            #[cfg(feature = "std")]
            Inner::Owned(msg) => msg.fmt(f),
            #[cfg(feature = "std")]
            Inner::Boxed(err) => err.fmt(f),
        }
    }
}

impl fmt::Display for Inner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Inner::Static(msg) => msg.fmt(f),
            #[cfg(feature = "std")]
            Inner::Owned(msg) => msg.fmt(f),
            #[cfg(feature = "std")]
            Inner::Boxed(err) => err.fmt(f),
        }
    }
}

#[cfg(feature = "std")]
impl error::Error for Inner {
    fn cause(&self) -> Option<&error::Error> {
        match self {
            Inner::Static(_) => None,
            Inner::Owned(_) => None,
            Inner::Boxed(err) => err.cause(),
        }
    }

    fn description(&self) -> &str {
        match self {
            Inner::Static(msg) => msg,
            Inner::Owned(msg) => msg,
            Inner::Boxed(err) => err.description(),
        }
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

impl AsRef<dyn error::Error + Send + Sync + 'static> for Error {
    fn as_ref(&self) -> &(dyn error::Error + Send + Sync + 'static) {
        self.as_error()
    }
}
