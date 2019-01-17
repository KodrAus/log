#[cfg(feature = "kv_serde")]
mod imp {
    use std::fmt;

    use crate::kv::value;

    impl<'v> value::Value<'v> {
        /// Create a value.
        pub fn from_serde(v: &'v (impl serde::Serialize + fmt::Debug)) -> Self {
            Self::from_any(v, |v, visit| visit.serde(v))
        }
    }

    impl<'v> serde::Serialize for value::Value<'v> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let mut visitor = SerdeBackend {
                serializer: Some(serializer),
                ok: None,
            };

            self.0.visit(value::Visitor(&mut visitor)).map_err(|_| <S::Error as serde::ser::Error>::custom("serialization failed"))?;

            Ok(visitor.ok.expect("missing return value"))
        }
    }

    impl<'a> value::Visitor<'a> {
        /// Visit a value that can be streamed with `serde`.
        pub fn serde(self, v: (impl serde::Serialize + fmt::Debug)) -> Result<(), value::Error> {
            self.0.serde(&v)
        }
    }

    /// The `serde` requirements for a backend.
    pub(in crate::kv::value) trait Backend {
        fn serde(&mut self, v: &dyn Value) -> Result<(), value::Error>;
    }

    /// An internal wrapper trait for `dyn erased_serde::Serialize + fmt::Debug`.
    pub(in crate::kv::value) trait Value: erased_serde::Serialize + fmt::Debug {}
    impl<T: ?Sized> Value for T where T: serde::Serialize + fmt::Debug {}

    impl<'a> serde::Serialize for &'a dyn Value {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            erased_serde::serialize(*self, serializer)
        }
    }

    // A visitor with a `serde` backend.
    struct SerdeBackend<S>
    where
        S: serde::Serializer,
    {
        serializer: Option<S>,
        ok: Option<S::Ok>,
    }

    impl<S> SerdeBackend<S>
    where
        S: serde::Serializer,
    {
        fn serialize(&mut self, v: impl erased_serde::Serialize) -> Result<(), value::Error> {
            self.ok = Some(erased_serde::serialize(&v, self.serializer.take().expect("missing serializer")).map_err(|_| value::Error::msg("serialization failed"))?);

            Ok(())
        }
    }

    impl<S> value::Backend for SerdeBackend<S>
    where
        S: serde::Serializer,
    {
        fn u64(&mut self, v: u64) -> Result<(), value::Error> {
            self.serde(&v)
        }

        fn i64(&mut self, v: i64) -> Result<(), value::Error> {
            self.serde(&v)
        }

        fn str(&mut self, v: &str) -> Result<(), value::Error> {
            self.serde(&v)
        }
    }

    impl<S> Backend for SerdeBackend<S>
    where
        S: serde::Serializer,
    {
        fn serde(&mut self, v: &dyn Value) -> Result<(), value::Error> {
            self.serialize(v)
        }
    }

    impl<S> value::fmt::Backend for SerdeBackend<S>
    where
        S: serde::Serializer,
    {
        fn debug(&mut self, v: &dyn fmt::Debug) -> Result<(), value::Error> {
            self.serialize(format_args!("{:?}", v))
        }
    }

    #[cfg(feature = "kv_sval")]
    impl<S> value::sval::Backend for SerdeBackend<S>
    where
        S: serde::Serializer,
    {
        fn sval(&mut self, v: &dyn value::sval::Value) -> Result<(), value::Error> {
            self.serialize(sval::serde::to_serialize(v))
        }
    }
}

#[cfg(not(feature = "kv_serde"))]
mod imp {
    use crate::kv::value;
    
    pub(in crate::kv::value) trait Backend {}

    impl<V: ?Sized> Backend for V where V: value::Backend {}
}

pub(super) use self::imp::*;