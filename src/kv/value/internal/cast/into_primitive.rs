// NOTE: With specialization we could potentially avoid this call using a blanket
// `ToPrimitive` trait that defaults to `None` except for these specific types
// It won't work with `&str` though in the `min_specialization` case
#[cfg(src_build)]
pub(in kv::value) fn into_primitive<'v>(value: &'v (dyn std::any::Any + 'static)) -> Option<crate::kv::value::internal::Primitive<'v>> {
    // The set of type ids that map to primitives are generated at build-time
    // by the contents of `sorted_type_ids.expr`. These type ids are pre-sorted
    // so that they can be searched efficiently. See the `sorted_type_ids.expr.rs`
    // file for the set of types that appear in this list
    const TYPE_IDS: [(std::any::TypeId, for<'a> fn(&'a (dyn std::any::Any + 'static)) -> crate::kv::value::internal::Primitive<'a>); 30] = include!(concat!(env!("OUT_DIR"), "/into_primitive.rs"));

    debug_assert!(TYPE_IDS.is_sorted_by_key(|&(k, _)| k));
    if let Ok(i) = TYPE_IDS.binary_search_by_key(&value.type_id(), |&(k, _)| k) {
        Some((TYPE_IDS[i].1)(value))
    } else {
        None
    }
}

// When the `src_build` config is not set then we're in the build script
// This function will generate an expression fragment used by `into_primitive`
#[cfg(not(src_build))]
pub fn generate() {
    use std::path::Path;
    use std::{fs, env};

    macro_rules! type_ids {
        ($($ty:ty,)*) => {
            [
                $(
                    (
                        std::any::TypeId::of::<$ty>(),
                        stringify!(
                            (
                                std::any::TypeId::of::<$ty>(),
                                (|value| unsafe {
                                    debug_assert_eq!(value.type_id(), std::any::TypeId::of::<$ty>());

                                    // SAFETY: We verify the value is $ty before casting
                                    let value = *(value as *const dyn std::any::Any as *const $ty);
                                    value.into()
                                }) as for<'a> fn(&'a (dyn std::any::Any + 'static)) -> crate::kv::value::internal::Primitive<'a>
                            )
                        )
                    ),
                )*
                $(
                    (
                        std::any::TypeId::of::<Option<$ty>>(),
                        stringify!(
                            (
                                std::any::TypeId::of::<Option<$ty>>(),
                                (|value| unsafe {
                                    debug_assert_eq!(value.type_id(), std::any::TypeId::of::<Option<$ty>>());

                                    // SAFETY: We verify the value is Option<$ty> before casting
                                    let value = *(value as *const dyn std::any::Any as *const Option<$ty>);
                                    if let Some(value) = value {
                                        value.into()
                                    } else {
                                        crate::kv::value::internal::Primitive::None
                                    }
                                }) as for<'a> fn(&'a (dyn std::any::Any + 'static)) -> crate::kv::value::internal::Primitive<'a>
                            )
                        )
                    ),
                )*
            ]
        };
    }

    let mut type_ids = type_ids![
        usize,
        u8,
        u16,
        u32,
        u64,

        isize,
        i8,
        i16,
        i32,
        i64,

        f32,
        f64,

        char,
        bool,

        &str,
    ];

    type_ids.sort_by_key(|&(k, _)| k);

    let mut ordered_type_ids_expr = String::new();

    ordered_type_ids_expr.push('[');

    for (_, v) in &type_ids {
        ordered_type_ids_expr.push_str(v);
        ordered_type_ids_expr.push(',');
    }

    ordered_type_ids_expr.push(']');

    let path = Path::new(&env::var_os("OUT_DIR").unwrap()).join("into_primitive.rs");
    fs::write(path, ordered_type_ids_expr).unwrap();
}
