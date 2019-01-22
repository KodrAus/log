use std::fmt;

/// An error encountered while visiting a key-value source.
pub struct Error(ErrorInner);

impl Error {
    /// Capture a static message as an error.
    pub fn msg(msg: &'static str) -> Self {
        Error(ErrorInner::Static(msg))
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

enum ErrorInner {
    Static(&'static str),
    #[cfg(feature = "std")]
    Owned(String),
}

impl fmt::Debug for ErrorInner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorInner::Static(msg) => msg.fmt(f),
            #[cfg(feature = "std")]
            ErrorInner::Owned(ref msg) => msg.fmt(f),
        }
    }
}

impl fmt::Display for ErrorInner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorInner::Static(msg) => msg.fmt(f),
            #[cfg(feature = "std")]
            ErrorInner::Owned(ref msg) => msg.fmt(f),
        }
    }
}

impl From<fmt::Error> for Error {
    #[cfg(feature = "std")]
    fn from(err: fmt::Error) -> Self {
        Self::custom(err)
    }

    #[cfg(not(feature = "std"))]
    fn from(_: fmt::Error) -> Self {
        Self::msg("formatting failed")
    }
}

impl From<Error> for fmt::Error {
    fn from(_: Error) -> Self {
        Self
    }
}

#[cfg(feature = "kv_sval")]
mod sval_support {
    use super::*;

    impl From<sval::Error> for Error {
        fn from(err: sval::Error) -> Self {
            Self::from_sval(err)
        }
    }

    #[cfg(not(feature = "std"))]
    impl Error {
        pub(crate) fn from_sval(err: sval::Error) -> Self {
            Error::msg("sval streaming failed")
        }

        /// Convert into `sval`.
        pub fn into_sval(self) -> sval::Error {
            sval::Error::msg("streaming failed")
        }
    }

    #[cfg(feature = "std")]
    impl Error {
        pub(crate) fn from_sval(err: sval::Error) -> Self {
            Error::custom(err)
        }

        /// Convert into `sval`.
        pub fn into_sval(self) -> sval::Error {
            self.into()
        }
    }
}

#[cfg(feature = "kv_serde")]
mod serde_support {
    use super::*;

    impl Error {
        /// Convert into `serde`.
        pub fn into_serde<E>(self) -> E
        where
            E: serde::ser::Error,
        {
            E::custom(self)
        }
    }

    
    impl Error {
        #[cfg(not(feature = "std"))]
        pub(crate) fn from_serde(err: impl serde::ser::Error) -> Self {
            Self::msg("serde serialization failed")
        }

        #[cfg(feature = "std")]
        pub(crate) fn from_serde(err: impl serde::ser::Error) -> Self {
            Self::custom(err)
        }
    }
}

#[cfg(feature = "std")]
mod std_support {
    use super::*;

    use std::{io, error};

    impl Error {
        /// Create an error for a formattable value.
        pub fn custom(err: impl fmt::Display) -> Self {
            Error(ErrorInner::Owned(err.to_string()))
        }
    }

    impl From<io::Error> for Error {
        fn from(err: io::Error) -> Self {
            Error::custom(err)
        }
    }

    impl From<Error> for io::Error {
        fn from(err: Error) -> Self {
            io::Error::new(io::ErrorKind::Other, err)
        }
    }

    impl error::Error for Error {
        fn description(&self) -> &str {
            self.0.description()
        }

        fn cause(&self) -> Option<&dyn error::Error> {
            self.0.cause()
        }
    }

    impl error::Error for ErrorInner {
        fn description(&self) -> &str {
            match self {
                ErrorInner::Static(msg) => msg,
                ErrorInner::Owned(msg) => msg,
            }
        }
    }
}
