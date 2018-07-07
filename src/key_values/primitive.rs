use std::{self, fmt};
use serde::ser::{Error, Serializer, Serialize, Impossible};

#[derive(Clone, Copy)]
pub struct Primitive(PrimitiveInner);

#[derive(Clone, Copy)]
pub enum PrimitiveInner {
    Unsigned(u64),
    Signed(i64),
    Float(f64),
    Bool(bool),
    Char(char),

    #[cfg(feature = "i128")]
    BigUnsigned(u128),
    
    #[cfg(feature = "i128")]
    BigSigned(i128),
}

impl Primitive {
    pub fn try_from<T>(v: T) -> Option<Self>
    where
        T: Serialize,
    {
        v.serialize(PrimitiveSerializer).ok()
    }
}

impl Serialize for Primitive {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.0 {
            PrimitiveInner::Unsigned(v) => v.serialize(serializer),
            PrimitiveInner::Signed(v) => v.serialize(serializer),
            PrimitiveInner::Float(v) => v.serialize(serializer),
            PrimitiveInner::Char(v) => v.serialize(serializer),
            PrimitiveInner::Bool(v) => v.serialize(serializer),

            #[cfg(feature = "i128")]
            PrimitiveInner::BigUnsigned(v) => v.serialize(serializer),
            #[cfg(feature = "i128")]
            PrimitiveInner::BigSigned(v) => v.serialize(serializer),
        }
    }
}

struct PrimitiveSerializer;

#[derive(Debug)]
struct Invalid;

impl Error for Invalid {
    fn custom<T>(_msg: T) -> Self
    where
        T: fmt::Display
    {
        Invalid
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Invalid {
    fn cause(&self) -> Option<&std::error::Error> {
        None
    }

    fn description(&self) -> &str {
        "invalid primitive"
    }
}

impl fmt::Display for Invalid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid primitive")
    }
}

impl Serializer for PrimitiveSerializer {
    type Ok = Primitive;
    type Error = Invalid;

    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, v: bool) -> Result<Primitive, Invalid> {
        Ok(Primitive(PrimitiveInner::Bool(v)))
    }

    fn serialize_i8(self, v: i8) -> Result<Primitive, Invalid> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i16(self, v: i16) -> Result<Primitive, Invalid> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i32(self, v: i32) -> Result<Primitive, Invalid> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i64(self, v: i64) -> Result<Primitive, Invalid> {
        Ok(Primitive(PrimitiveInner::Signed(v)))
    }

    fn serialize_u8(self, v: u8) -> Result<Primitive, Invalid> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u16(self, v: u16) -> Result<Primitive, Invalid> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u32(self, v: u32) -> Result<Primitive, Invalid> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u64(self, v: u64) -> Result<Primitive, Invalid> {
        Ok(Primitive(PrimitiveInner::Unsigned(v)))
    }

    serde_if_integer128! {
        #[cfg(feature = "i128")]
        fn serialize_u128(self, v: u128) -> Result<Primitive, Invalid> {
            Ok(Primitive(PrimitiveInner::BigUnsigned(v)))
        }

        #[cfg(feature = "i128")]
        fn serialize_i128(self, v: i128) -> Result<Primitive, Invalid> {
            Ok(Primitive(PrimitiveInner::BigSigned(v)))
        }
    }

    fn serialize_f32(self, v: f32) -> Result<Primitive, Invalid> {
        self.serialize_f64(v as f64)
    }

    fn serialize_f64(self, v: f64) -> Result<Primitive, Invalid> {
        Ok(Primitive(PrimitiveInner::Float(v)))
    }

    fn serialize_char(self, v: char) -> Result<Primitive, Invalid> {
        Ok(Primitive(PrimitiveInner::Char(v)))
    }

    fn serialize_str(self, v: &str) -> Result<Primitive, Invalid> {
        Err(Invalid)
    }

    fn collect_str<T: fmt::Display + ?Sized>(self, v: &T) -> Result<Primitive, Invalid> {
        Err(Invalid)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Primitive, Invalid> {
        Err(Invalid)
    }

    fn serialize_none(self) -> Result<Primitive, Invalid> {
        Err(Invalid)
    }

    fn serialize_some<T>(self, value: &T) -> Result<Primitive, Invalid>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Primitive, Invalid> {
        Err(Invalid)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Primitive, Invalid> {
        Err(Invalid)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Primitive, Invalid> {
        Err(Invalid)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Primitive, Invalid>
    where
        T: ?Sized + Serialize,
    {
        Err(Invalid)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Primitive, Invalid>
    where
        T: ?Sized + Serialize,
    {
        Err(Invalid)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(Invalid)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(Invalid)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(Invalid)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Invalid)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(Invalid)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(Invalid)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(Invalid)
    }
}
