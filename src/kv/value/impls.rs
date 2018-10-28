use super::*;

/// This trait is a private implementation detail for testing.
/// 
/// All it does is make sure that our set of concrete types
/// that implement `Visit` always implement the `Visit` trait,
/// regardless of crate features and blanket implementations.
trait EnsureVisit: Visit {}

impl<'a, T> EnsureVisit for &'a T where T: Visit {}

macro_rules! impl_to_value {
    () => {};
    (
        impl: { $($params:tt)* }
        where: { $($where:tt)* }
        $ty:ty: { $($serialize:tt)* }
        $($rest:tt)*
    ) => {
        impl<$($params)*> EnsureVisit for $ty
        where
            $($where)* {}
        
        #[cfg(not(feature = "kv_serde"))]
        impl<$($params)*> private::Sealed for $ty
        where
            $($where)* {}

        #[cfg(not(feature = "kv_serde"))]
        impl<$($params)*> Visit for $ty
        where
            $($where)*
        {
            $($serialize)*
        }

        impl_to_value!($($rest)*);
    };
    (
        impl: { $($params:tt)* }
        $ty:ty: { $($serialize:tt)* } 
        $($rest:tt)*
    ) => {
        impl_to_value! {
            impl: {$($params)*} where: {} $ty: { $($serialize)* } $($rest)*
        }
    };
    (
        $ty:ty: { $($serialize:tt)* } 
        $($rest:tt)*
    ) => {
        impl_to_value! {
            impl: {} where: {} $ty: { $($serialize)* } $($rest)*
        }
    }
}

impl_to_value! {
    u8: {
        fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
            visitor.visit_u64(*self as u64)
        }
    }
    u16: {
        fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
            visitor.visit_u64(*self as u64)
        }
    }
    u32: {
        fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
            visitor.visit_u64(*self as u64)
        }
    }
    u64: {
        fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
            visitor.visit_u64(*self)
        }
    }

    i8: {
        fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
            visitor.visit_i64(*self as i64)
        }
    }
    i16: {
        fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
            visitor.visit_i64(*self as i64)
        }
    }
    i32: {
        fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
            visitor.visit_i64(*self as i64)
        }
    }
    i64: {
        fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
            visitor.visit_i64(*self)
        }
    }

    f32: {
        fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
            visitor.visit_f64(*self as f64)
        }
    }
    f64: {
        fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
            visitor.visit_f64(*self)
        }
    }

    char: {
        fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
            visitor.visit_char(*self)
        }
    }

    bool: {
        fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
            visitor.visit_bool(*self)
        }
    }
    
    (): {
        fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
            visitor.visit_none()
        }
    }

    impl: { 'a } &'a str: {
        fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
            visitor.visit_str(self)
        }
    }

    impl: { 'a } &'a [u8]: {
        fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
            visitor.visit_bytes(self)
        }
    }

    impl: { 'a } fmt::Arguments<'a>: {
        fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
            visitor.visit_fmt(self)
        }
    }

    impl: { 'v } Value<'v>: {
        fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
            self.visit(visitor)
        }
    }
    
    impl: { T: Visit } Option<T>: {
        fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
            match self {
                Some(v) => v.to_value().visit(visitor),
                None => visitor.visit_none(),
            }
        }
    }
}

#[cfg(feature = "i128")]
impl_to_value! {
    u128: {
        fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
            visitor.visit_u128(*self)
        }
    }
    i128: {
        fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
            visitor.visit_i128(*self)
        }
    }
}

#[cfg(feature = "std")]
mod std_support {
    use super::*;

    use std::path::{Path, PathBuf};

    impl_to_value! {
        String: {
            fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
                visitor.visit_str(&*self)
            }
        }

        Vec<u8>: {
            fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
                visitor.visit_bytes(&*self)
            }
        }

        impl: { 'a } &'a Path: {
            fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
                match self.to_str() {
                    Some(s) => visitor.visit_str(s),
                    None => visitor.visit_fmt(&format_args!("{:?}", self)),
                }
            }
        }
        PathBuf: {
            fn visit(&self, visitor: &mut dyn Visitor) -> Result<(), Error> {
                self.as_path().visit(visitor)
            }
        }
    }
}
