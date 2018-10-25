//! Serialization for structured values.

use std::fmt;

#[doc(inline)]
pub use super::Error;

/// An arbitrary structured value.
/// 
/// **This trait cannot be implemented manually**
/// 
/// The `Visit` trait is always implemented for a fixed set of primitives:
/// 
/// - Standard formats: `Arguments`
/// - Primitives: `bool`, `char`
/// - Unsigned integers: `u8`, `u16`, `u32`, `u64`, `u128`
/// - Signed integers: `i8`, `i16`, `i32`, `i64`, `i128`
/// - Strings: `&str`, `String`
/// - Bytes: `&[u8]`, `Vec<u8>`
/// - Paths: `Path`, `PathBuf`
/// 
/// Any other type that implements `serde::Serialize + std::fmt::Debug` will
/// automatically implement `Visit` if the `structured_serde` feature is
/// enabled.
pub trait Value: fmt::Debug + visit_imp::ValuePrivate {
    /// Visit the value with the given serializer.
    fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error>;
}

/// A serializer for primitive values.
pub trait Visitor {
    /// Visit an arbitrary value.
    /// 
    /// Depending on crate features there are a few things
    /// you can do with a value. You can:
    /// 
    /// - format it using `Debug`.
    /// - serialize it using `serde`.
    fn visit_any(&mut self, v: &dyn Value) -> Result<(), Error>;

    /// Visit a signed integer.
    fn visit_i64(&mut self, v: i64) -> Result<(), Error> {
        self.visit_any(&v)
    }

    /// Visit an unsigned integer.
    fn visit_u64(&mut self, v: u64) -> Result<(), Error> {
        self.visit_any(&v)
    }

    /// Visit a floating point number.
    fn visit_f64(&mut self, v: f64) -> Result<(), Error> {
        self.visit_any(&v)
    }

    /// Visit a boolean.
    fn visit_bool(&mut self, v: bool) -> Result<(), Error> {
        self.visit_any(&v)
    }

    /// Visit a single character.
    fn visit_char(&mut self, v: char) -> Result<(), Error> {
        let mut b = [0; 4];
        self.visit_str(&*v.encode_utf8(&mut b))
    }

    /// Visit a UTF8 string.
    fn visit_str(&mut self, v: &str) -> Result<(), Error> {
        self.visit_any(&v)
    }

    /// Visit a raw byte buffer.
    fn visit_bytes(&mut self, v: &[u8]) -> Result<(), Error> {
        self.visit_any(&v)
    }

    /// Visit standard arguments.
    fn visit_fmt(&mut self, v: &fmt::Arguments) -> Result<(), Error> {
        self.visit_any(&v)
    }
}

impl<'a, T: ?Sized> Visitor for &'a mut T
where
    T: Visitor,
{
    fn visit_any(&mut self, v: &dyn Value) -> Result<(), Error> {
        (**self).visit_any(v)
    }

    fn visit_i64(&mut self, v: i64) -> Result<(), Error> {
        (**self).visit_i64(v)
    }

    fn visit_u64(&mut self, v: u64) -> Result<(), Error> {
        (**self).visit_u64(v)
    }

    fn visit_f64(&mut self, v: f64) -> Result<(), Error> {
        (**self).visit_f64(v)
    }

    fn visit_bool(&mut self, v: bool) -> Result<(), Error> {
        (**self).visit_bool(v)
    }

    fn visit_char(&mut self, v: char) -> Result<(), Error> {
        (**self).visit_char(v)
    }

    fn visit_str(&mut self, v: &str) -> Result<(), Error> {
        (**self).visit_str(v)
    }

    fn visit_bytes(&mut self, v: &[u8]) -> Result<(), Error> {
        (**self).visit_bytes(v)
    }

    fn visit_fmt(&mut self, args: &fmt::Arguments) -> Result<(), Error> {
        (**self).visit_fmt(args)
    }
}

/// This trait is a private implementation detail for testing.
/// 
/// All it does is make sure that our set of concrete types
/// that implement `Visit` always implement the `Visit` trait,
/// regardless of crate features and blanket implementations.
trait EnsureValue: Value {}

macro_rules! ensure_impl_visit {
    ($(<$($params:tt),*> $ty:ty { $($serialize:tt)* })*) => {
        $(
            impl<$($params),*> EnsureValue for $ty {}
            impl<'ensure_visit, $($params),*> EnsureValue for &'ensure_visit $ty {}

            #[cfg(not(feature = "structured_serde"))]
            impl<$($params),*> Value for $ty {
                $($serialize)*
            }

            #[cfg(not(feature = "structured_serde"))]
            impl<$($params),*> visit_imp::ValuePrivate for $ty {}
        )*
    };
    ($($ty:ty { $($serialize:tt)* })*) => {
        $(
            impl EnsureValue for $ty {}
            impl<'ensure_visit> EnsureValue for &'ensure_visit $ty {}

            #[cfg(not(feature = "structured_serde"))]
            impl Value for $ty {
                $($serialize)*
            }

            #[cfg(not(feature = "structured_serde"))]
            impl visit_imp::ValuePrivate for $ty {}
        )*
    }
}

ensure_impl_visit! {
    u8 {
        fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
            visitor.visit_u64(*self as u64)
        }
    }
    u16 {
        fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
            visitor.visit_u64(*self as u64)
        }
    }
    u32 {
        fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
            visitor.visit_u64(*self as u64)
        }
    }
    u64 {
        fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
            visitor.visit_u64(*self)
        }
    }

    i8 {
        fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
            visitor.visit_i64(*self as i64)
        }
    }
    i16 {
        fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
            visitor.visit_i64(*self as i64)
        }
    }
    i32 {
        fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
            visitor.visit_i64(*self as i64)
        }
    }
    i64 {
        fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
            visitor.visit_i64(*self)
        }
    }

    f32 {
        fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
            visitor.visit_f64(*self as f64)
        }
    }
    f64 {
        fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
            visitor.visit_f64(*self)
        }
    }

    char {
        fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
            visitor.visit_char(*self)
        }
    }
    bool {
        fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
            visitor.visit_bool(*self)
        }
    }
    str {
        fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
            visitor.visit_str(self)
        }
    }
    [u8] {
        fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
            visitor.visit_bytes(self)
        }
    }
}

ensure_impl_visit! {
    <'a> fmt::Arguments<'a> {
        fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
            visitor.visit_fmt(self)
        }
    }

    <'v> super::source::Value<'v> {
        fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
            use super::private::{ValueInner, value_inner};

            match value_inner(self) {
                ValueInner::Borrowed(v) => v.visit(visitor),
                ValueInner::Any(v) => v.visit(visitor),
                #[cfg(feature = "std")]
                ValueInner::Owned(ref v) => v.visit(visitor),
            }
        }
    }
}

impl EnsureValue for dyn Value {}

#[cfg(not(feature = "structured_serde"))]
mod visit_imp {
    use super::*;

    #[doc(hidden)]
    pub trait ValuePrivate {}

    impl<'a, T: ?Sized> Value for &'a T
    where
        T: Value,
    {
        fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
            (**self).visit(visitor)
        }
    }

    impl<'a, T: ?Sized> ValuePrivate for &'a T
    where
        T: Value,
    {
    }
}

#[cfg(feature = "structured_serde")]
mod visit_imp {
    use super::*;

    use erased_serde;
    use serde::{Serialize, Serializer};

    #[doc(hidden)]
    pub trait ValuePrivate: erased_serde::Serialize {}
 
    impl<T: ?Sized> Value for T
    where
        T: Serialize + fmt::Debug,
    {
        fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
            match Serialize::serialize(self, SerdeBridge(visitor)) {
                Err(SerdeError::Unsupported) => visitor.visit_fmt(&format_args!("{:?}", self)),
                Err(SerdeError::Other(e)) => Err(e),
                Ok(()) => Ok(())
            }
        }
    }

    impl<T: ?Sized> ValuePrivate for T
    where
        T: Serialize + fmt::Debug,
    {
    }

    struct SerdeBridge<'a>(&'a mut dyn Visitor);

    #[derive(Debug)]
    enum SerdeError {
        Unsupported,
        Other(Error),
    }

    impl serde::ser::Error for SerdeError {
        fn custom<T>(_msg: T) -> Self
        where
            T: std::fmt::Display
        {
            SerdeError::Unsupported
        }
    }

    impl std::fmt::Display for SerdeError {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "serialization error")
        }
    }

    impl std::error::Error for SerdeError {
        fn cause(&self) -> Option<&dyn std::error::Error> {
            None
        }

        fn description(&self) -> &str {
            "serialization error"
        }
    }

    impl<'a> Serializer for SerdeBridge<'a> {
        type Ok = ();
        type Error = SerdeError;

        type SerializeSeq = serde::ser::Impossible<Self::Ok, Self::Error>;
        type SerializeTuple = serde::ser::Impossible<Self::Ok, Self::Error>;
        type SerializeTupleStruct = serde::ser::Impossible<Self::Ok, Self::Error>;
        type SerializeTupleVariant = serde::ser::Impossible<Self::Ok, Self::Error>;
        type SerializeMap = serde::ser::Impossible<Self::Ok, Self::Error>;
        type SerializeStruct = serde::ser::Impossible<Self::Ok, Self::Error>;
        type SerializeStructVariant = serde::ser::Impossible<Self::Ok, Self::Error>;

        fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
            self.0.visit_bool(v).map_err(SerdeError::Other)
        }

        fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
            self.serialize_i64(v as i64)
        }

        fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
            self.serialize_i64(v as i64)
        }

        fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
            self.serialize_i64(v as i64)
        }

        fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
            self.0.visit_i64(v).map_err(SerdeError::Other)
        }

        fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
            self.serialize_u64(v as u64)
        }

        fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
            self.serialize_u64(v as u64)
        }

        fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
            self.serialize_u64(v as u64)
        }

        fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
            self.0.visit_u64(v).map_err(SerdeError::Other)
        }

        fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
            self.serialize_f64(v as f64)
        }

        fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
            self.0.visit_f64(v).map_err(SerdeError::Other)
        }

        fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
            self.0.visit_char(v).map_err(SerdeError::Other)
        }

        fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
            self.0.visit_str(v).map_err(SerdeError::Other)
        }

        fn collect_str<T: std::fmt::Display + ?Sized>(self, v: &T) -> Result<Self::Ok, Self::Error> {
            self.0.visit_fmt(&format_args!("{}", v)).map_err(SerdeError::Other)
        }

        fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
            self.0.visit_bytes(v).map_err(SerdeError::Other)
        }

        fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
            Err(SerdeError::Unsupported)
        }

        fn serialize_some<T>(self, v: &T) -> Result<Self::Ok, Self::Error>
        where
            T: ?Sized + Serialize,
        {
            v.serialize(self)
        }

        fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
            Err(SerdeError::Unsupported)
        }

        fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
            Err(SerdeError::Unsupported)
        }

        fn serialize_unit_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            _variant: &'static str,
        ) -> Result<Self::Ok, Self::Error> {
            Err(SerdeError::Unsupported)
        }

        fn serialize_newtype_struct<T>(
            self,
            _name: &'static str,
            _value: &T,
        ) -> Result<Self::Ok, Self::Error>
        where
            T: ?Sized + Serialize,
        {
            Err(SerdeError::Unsupported)
        }

        fn serialize_newtype_variant<T>(
            self,
            _name: &'static str,
            _variant_index: u32,
            _variant: &'static str,
            _value: &T,
        ) -> Result<Self::Ok, Self::Error>
        where
            T: ?Sized + Serialize,
        {
            Err(SerdeError::Unsupported)
        }

        fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
            Err(SerdeError::Unsupported)
        }

        fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
            Err(SerdeError::Unsupported)
        }

        fn serialize_tuple_struct(
            self,
            _name: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeTupleStruct, Self::Error> {
            Err(SerdeError::Unsupported)
        }

        fn serialize_tuple_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            _variant: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeTupleVariant, Self::Error> {
            Err(SerdeError::Unsupported)
        }

        fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
            Err(SerdeError::Unsupported)
        }

        fn serialize_struct(
            self,
            _name: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeStruct, Self::Error> {
            Err(SerdeError::Unsupported)
        }

        fn serialize_struct_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            _variant: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeStructVariant, Self::Error> {
            Err(SerdeError::Unsupported)
        }
    }
}

#[cfg(feature = "structured_serde")]
mod serde_support {
    use super::*;

    use serde::{Serialize, Serializer};

    impl<'a> Serialize for dyn Value + 'a {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            erased_serde::serialize(self, serializer)
        }
    }
}

#[cfg(feature = "structured_serde")]
pub use self::serde_support::*;

#[cfg(feature = "std")]
mod std_support {
    use super::*;

    use std::path::{Path, PathBuf};

    ensure_impl_visit! {
        String {
            fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
                visitor.visit_str(&*self)
            }
        }
        Vec<u8> {
            fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
                visitor.visit_bytes(&*self)
            }
        }
        Path {
            fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
                match self.to_str() {
                    Some(s) => visitor.visit_str(s),
                    None => visitor.visit_fmt(&format_args!("{:?}", self)),
                }
            }
        }
        PathBuf {
            fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
                self.as_path().visit(visitor)
            }
        }
    }
}

#[cfg(feature = "std")]
pub use self::std_support::*;

#[cfg(test)]
mod tests {
    use crate::*;

    #[derive(PartialEq, Debug)]
    enum Token<'a> {
        I64(i64),
        U64(u64),
        F64(f64),
        Bool(bool),
        Char(char),
        Str(&'a str),
        Bytes(&'a [u8]),
        Args(&'a str),
    }

    // `&dyn ser::Serialize` should impl `Serialize`
    fn assert_visit(v: &dyn Value, token: Token) {
        struct TestVisitor<'a>(Token<'a>);

        impl<'a> Visitor for TestVisitor<'a> {
            fn visit_i64(&mut self, v: i64) {
                assert_eq!(self.0, Token::I64(v));
            }
            
            fn visit_u64(&mut self, v: u64) {
                assert_eq!(self.0, Token::U64(v));
            }

            fn visit_f64(&mut self, v: f64) {
                assert_eq!(self.0, Token::F64(v));
            }

            fn visit_bool(&mut self, v: bool) {
                assert_eq!(self.0, Token::Bool(v));
            }

            fn visit_char(&mut self, v: char) {
                assert_eq!(self.0, Token::Char(v));
            }

            fn visit_str(&mut self, v: &str) {
                assert_eq!(self.0, Token::Str(v));
            }

            fn visit_bytes(&mut self, v: &[u8]) {
                assert_eq!(self.0, Token::Bytes(v));
            }

            fn visit_fmt(&mut self, v: &fmt::Arguments) {
                use self::std::{str, ptr};
                use self::fmt::Write;

                const LEN: usize = 128;

                struct ValueArgs {
                    buf: [u8; LEN],
                    cursor: usize,
                }

                impl ValueArgs {
                    fn new() -> Self {
                        ValueArgs {
                            buf: [0; LEN],
                            cursor: 0,
                        }
                    }

                    fn to_str(&self) -> Option<&str> {
                        str::from_utf8(&self.buf[0..self.cursor]).ok()
                    }
                }

                impl Write for ValueArgs {
                    fn write_str(&mut self, s: &str) -> fmt::Result {
                        let src = s.as_bytes();
                        let next_cursor = self.cursor + src.len();

                        if next_cursor > LEN {
                            return Err(fmt::Error);
                        }

                        unsafe {
                            let src_ptr = src.as_ptr();
                            let dst_ptr = self.buf.as_mut_ptr().offset(self.cursor as isize);

                            ptr::copy_nonoverlapping(src_ptr, dst_ptr, src.len());
                        }

                        self.cursor = next_cursor;

                        Ok(())
                    }
                }

                let mut w = ValueArgs::new();
                w.write_fmt(format_args!("{}", v)).unwrap();
                assert_eq!(self.0, Token::Args(w.to_str().unwrap()));
            }
        }

        v.visit(&mut TestVisitor(token));
    }

    #[test]
    fn visit_simple() {
        assert_visit(&1u8, Token::U64(1u64));
        assert_visit(&true, Token::Bool(true));
        assert_visit(&"a string", Token::Str("a string"));
    }

    #[test]
    #[cfg(feature = "structured_serde")]
    fn visit_unsupported_as_debug() {
        use serde_json::json;

        let v = json!({
            "id": 123,
            "name": "alice",
        });

        assert_visit(&v, Token::Args(&format!("{:?}", v)));
    }

    #[cfg(feature = "structured_serde")]
    mod structured_serde {
        use crate::*;
        use serde_test::{Token, assert_ser_tokens};
        use serde_json::json;

        // `&dyn ser::Serialize` should impl `Serialize`
        fn assert_visit(v: &dyn Value, tokens: &[Token]) {
            assert_ser_tokens(&v, tokens);
        }

        #[test]
        fn visit_simple() {
            assert_visit(&1u8, &[Token::U8(1u8)]);
            assert_visit(&true, &[Token::Bool(true)]);
            assert_visit(&"a string", &[Token::Str("a string")]);
        }

        #[test]
        fn visit_complex() {
            let v = json!({
                "id": 123,
                "name": "alice",
            });

            assert_visit(&v, &[
                Token::Map { len: Some(2) },
                Token::Str("id"),
                Token::U64(123),
                Token::Str("name"),
                Token::Str("alice"),
                Token::MapEnd,
            ]);
        }
    }
}
