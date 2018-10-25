//! Structured key-value pairs for log records.

#[macro_use]
mod macros;

mod error;
pub mod value;
pub mod source;

pub use self::error::Error;

#[doc(inline)]
pub use self::source::Source;

mod private {
    use std::fmt;
    use std::mem;
    use std::marker::PhantomData;
    use std::cmp::Ordering;
    use std::hash::{Hash, Hasher};
    use std::borrow::Borrow;

    use super::{value, Error};

    /// The key in a key-value pair.
    /// 
    /// The `Key` type abstracts over owned or borrowed keys.
    pub struct Key<'k>(KeyInner<'k>);

    impl<'k> Key<'k> {
        pub fn new(k: &'k (impl Borrow<str> + ?Sized)) -> Self {
            Key(KeyInner::Borrowed(k.borrow()))
        }

        pub fn as_str(&self) -> &str {
            match self.0 {
                KeyInner::Borrowed(k) => k,
                #[cfg(feature = "std")]
                KeyInner::Owned(ref k) => &*k,
            }
        }
    }

    impl<'k> AsRef<str> for Key<'k> {
        fn as_ref(&self) -> &str {
            self.as_str()
        }
    }

    impl<'k> Borrow<str> for Key<'k> {
        fn borrow(&self) -> &str {
            self.as_str()
        }
    }

    impl<'k> From<&'k str> for Key<'k> {
        fn from(k: &'k str) -> Self {
            Key(KeyInner::Borrowed(k))
        }
    }

    impl<'k> PartialEq for Key<'k> {
        fn eq(&self, other: &Self) -> bool {
            self.as_str().eq(other.as_str())
        }
    }

    impl<'k> Eq for Key<'k> {}

    impl<'k> PartialOrd for Key<'k> {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            self.as_str().partial_cmp(other.as_str())
        }
    }

    impl<'k> Ord for Key<'k> {
        fn cmp(&self, other: &Self) -> Ordering {
            self.as_str().cmp(other.as_str())
        }
    }

    impl<'k> Hash for Key<'k> {
        fn hash<H>(&self, state: &mut H)
        where
            H: Hasher,
        {
            self.as_str().hash(state)
        }
    }

    impl<'k> fmt::Debug for Key<'k> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            self.as_str().fmt(f)
        }
    }

    impl<'k> fmt::Display for Key<'k> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            self.as_str().fmt(f)
        }
    }

    /// The value in a key-value pair.
    pub struct Value<'v>(ValueInner<'v>);

    impl<'v> Value<'v> {
        /// Create a value from a borrowed trait object.
        pub fn new(v: &'v impl value::Value) -> Self {
            Value(ValueInner::Borrowed(v))
        }

        /// Create a value from an anonymous value.
        /// 
        /// The value must be provided with a compatible visit method.
        pub fn any<T>(v: &'v T, visit: fn(&T, &mut dyn value::Visitor) -> Result<(), Error>) -> Self
        where
            T: 'static,
        {
            Value(ValueInner::Any(Any::new(v, visit)))
        }
    }

    impl<'v> fmt::Debug for Value<'v> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self.0 {
                ValueInner::Borrowed(v) => v.fmt(f),
                ValueInner::Any(ref v) => {
                    struct Visitor<'a, 'b>(&'a mut fmt::Formatter<'b>);

                    impl<'a, 'b> value::Visitor for Visitor<'a, 'b> {
                        fn visit_any(&mut self, v: &dyn value::Value) -> Result<(), Error> {
                            write!(self.0, "{:?}", v).map_err(|_| Error::msg("formatting failed"))?;

                            Ok(())
                        }
                    }

                    let mut visitor = Visitor(f);
                    v.visit(&mut visitor).map_err(|_| fmt::Error)
                }
                #[cfg(feature = "std")]
                ValueInner::Owned(ref v) => v.fmt(f),
            }
        }
    }


    // pub(super)

    pub enum KeyInner<'k> {
        Borrowed(&'k str),
        #[cfg(feature = "std")]
        Owned(String),
    }

    pub enum ValueInner<'v> {
        Borrowed(&'v dyn value::Value),
        Any(Any<'v>),
        #[cfg(feature = "std")]
        Owned(Box<dyn value::Value>),
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

    pub struct Any<'a> {
        data: &'a Void,
        visit: fn(&Void, &mut dyn value::Visitor) -> Result<(), Error>,
    }

    impl<'a> Any<'a> {
        pub fn new<T>(data: &'a T, visit: fn(&T, &mut dyn value::Visitor) -> Result<(), Error>) -> Self
        where
            T: 'static,
        {
            unsafe {
                Any {
                    data: mem::transmute::<&'a T, &'a Void>(data),
                    visit: mem::transmute::<
                        fn(&T, &mut dyn value::Visitor) -> Result<(), Error>,
                        fn(&Void, &mut dyn value::Visitor) -> Result<(), Error>>
                        (visit),
                }
            }
        }

        pub fn visit(&self, visitor: &mut dyn value::Visitor) -> Result<(), Error> {
            (self.visit)(self.data, visitor)
        }
    }

    #[cfg(not(feature = "kv_serde"))]
    pub fn value_inner<'a, 'b>(v: &'a Value<'b>) -> &'a ValueInner<'b> { &v.0 }

    #[cfg(feature = "std")]
    mod std_support {
        use super::*;

        impl<'k> Key<'k> {
            pub fn owned(k: impl Into<String>) -> Self {
                Key(KeyInner::Owned(k.into()))
            }
        }

        impl<'k> From<String> for Key<'k> {
            fn from(k: String) -> Self {
                Key::owned(k)
            }
        }

        impl<'v> Value<'v> {
            pub fn owned(v: impl value::Value + 'static) -> Self {
                Value(ValueInner::Owned(Box::new(v)))
            }
        }
    }

    #[cfg(feature = "kv_serde")]
    mod serde_support {
        use super::*;

        use serde::{Serialize, Serializer};

        impl<'k> Serialize for Key<'k> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                self.as_str().serialize(serializer)
            }
        }

        impl<'v> Serialize for Value<'v> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                match self.0 {
                    ValueInner::Any(ref v) => {
                        struct Visitor<S: Serializer> {
                            serializer: Option<S>,
                            ok: Option<S::Ok>,
                        }

                        impl<S> value::Visitor for Visitor<S>
                        where
                            S: Serializer,
                        {
                            fn visit_any(&mut self, v: &dyn value::Value) -> Result<(), Error> {
                                let ok = v.serialize(self.serializer.take().expect("missing serializer"))?;
                                self.ok = Some(ok);

                                Ok(())
                            }
                        }

                        let mut visitor = Visitor {
                            serializer: Some(serializer),
                            ok: None,
                        };

                        v.visit(&mut visitor).map_err(|e| e.into_serde())?;
                        Ok(visitor.ok.expect("missing return value"))
                    },
                    ValueInner::Borrowed(v) => v.serialize(serializer),
                    ValueInner::Owned(ref v) => v.serialize(serializer),
                }
            }
        }
    }
}