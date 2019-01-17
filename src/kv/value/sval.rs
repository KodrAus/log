#[cfg(feature = "kv_sval")]
mod imp {
    use std::fmt;

    use crate::kv::value;

    impl<'v> value::Value<'v> {
        /// Create a value.
        pub fn from_sval(v: &'v (impl sval::Value + fmt::Debug)) -> Self {
            Self::from_any(v, |v, visit| visit.sval(v))
        }
    }

    impl<'v> sval::Value for value::Value<'v> {
        fn stream(&self, stream: &mut sval::value::Stream) -> Result<(), sval::value::Error> {
            let mut visitor = SvalBackend(stream);
            self.0.visit(value::Visitor(&mut visitor)).map_err(|_| sval::value::Error::msg("serialization failed"))
        }
    }

    impl<'a> value::Visitor<'a> {
        /// Visit a value that can be streamed with `sval`.
        pub fn sval(self, v: (impl sval::Value + fmt::Debug)) -> Result<(), value::Error> {
            self.0.sval(&v)
        }
    }

    /// The `sval` requirements for a backend.
    pub(in crate::kv::value) trait Backend {
        fn sval(&mut self, v: &dyn Value) -> Result<(), value::Error>;
    }

    /// An internal wrapper trait for `dyn sval::Value + fmt::Debug`.
    pub(in crate::kv::value) trait Value: sval::Value + fmt::Debug {}
    impl<T: ?Sized> Value for T where T: sval::Value + fmt::Debug {}

    // A visitor with an `sval` backend.
    pub(in crate::kv::value) struct SvalBackend<'a, 'b>(&'a mut sval::value::Stream<'b>);

    impl<'a, 'b> SvalBackend<'a, 'b> {
        fn any(&mut self, v: impl sval::Value) -> Result<(), value::Error> {
            self.0.any(v).map_err(|_| value::Error::msg("serialization failed"))?;

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
        fn debug(&mut self, v: &dyn fmt::Debug) -> Result<(), value::Error> {
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
    use crate::kv::value;
    
    pub(in crate::kv::value) trait Backend {}

    impl<V: ?Sized> Backend for V where V: value::Backend {}
}

pub(super) use self::imp::*;