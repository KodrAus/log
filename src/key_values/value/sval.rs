/*
An `sval` backend for structured values.

This module allows capturing `impl sval::Value` as a `Value`,
and streaming any `Value` using `sval`.
*/

#[cfg(feature = "kv_sval")]
mod imp {
    use std::fmt;

    use crate::key_values::value;

    impl<'v> value::Value<'v> {
        /// Create a value from a `sval::Value`.
        pub fn from_sval(v: &'v (impl sval::Value + fmt::Debug)) -> Self {
            Self::from_any(v, |from, v| from.sval(v))
        }
    }

    impl<'v> sval::Value for value::Value<'v> {
        fn stream(&self, stream: &mut sval::value::Stream) -> Result<(), sval::value::Error> {
            self.0.visit(&mut SvalBackend(stream))?;

            Ok(())
        }
    }

    impl<'a> value::FromAny<'a> {
        /// Visit a value that can be streamed with `sval`.
        pub fn sval(self, v: (impl sval::Value + fmt::Debug)) -> Result<(), value::Error> {
            self.0.sval(&v)
        }
    }

    /// The `sval` requirements for a backend.
    pub(in crate::key_values::value) trait Backend {
        fn sval(&mut self, v: &dyn Value) -> Result<(), value::Error>;
    }

    /// An internal wrapper trait for `dyn sval::Value + fmt::Debug`.
    pub(in crate::key_values::value) trait Value: sval::Value + fmt::Debug {}
    impl<T: ?Sized> Value for T where T: sval::Value + fmt::Debug {}

    // A visitor with an `sval` backend.
    struct SvalBackend<'a, 'b>(&'a mut sval::value::Stream<'b>);

    impl<'a, 'b> SvalBackend<'a, 'b> {
        fn any(&mut self, v: impl sval::Value) -> Result<(), value::Error> {
            self.0.any(v)?;

            Ok(())
        }
    }

    impl<'a, 'b> value::Backend for SvalBackend<'a, 'b> {
        fn u64(&mut self, v: u64) -> Result<(), value::Error> {
            self.sval(&v)
        }

        fn i64(&mut self, v: i64) -> Result<(), value::Error> {
            self.sval(&v)
        }

        fn f64(&mut self, v: f64) -> Result<(), value::Error> {
            self.sval(&v)
        }

        fn bool(&mut self, v: bool) -> Result<(), value::Error> {
            self.sval(&v)
        }

        fn char(&mut self, v: char) -> Result<(), value::Error> {
            self.sval(&v)
        }

        fn none(&mut self) -> Result<(), value::Error> {
            self.sval(&Option::None::<()>)
        }

        fn str(&mut self, v: &str) -> Result<(), value::Error> {
            self.sval(&v)
        }
    }

    impl<'a, 'b> Backend for SvalBackend<'a, 'b> {
        fn sval(&mut self, v: &dyn Value) -> Result<(), value::Error> {
            self.any(v)
        }
    }

    impl<'a, 'b> value::fmt::Backend for SvalBackend<'a, 'b> {
        fn debug(&mut self, v: &dyn value::fmt::Value) -> Result<(), value::Error> {
            self.any(format_args!("{:?}", v))
        }
    }

    #[cfg(feature = "kv_serde")]
    impl<'a, 'b> value::serde::Backend for SvalBackend<'a, 'b> {
        fn serde(&mut self, v: &dyn value::serde::Value) -> Result<(), value::Error> {
            self.any(sval::serde::to_value(v))
        }
    }
}

#[cfg(not(feature = "kv_sval"))]
mod imp {
    use crate::key_values::value;
    
    pub(in crate::key_values::value) trait Backend {}

    impl<V: ?Sized> Backend for V where V: value::Backend {}
}

pub(super) use self::imp::*;