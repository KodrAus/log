#[macro_export]
#[doc(hidden)]
macro_rules! properties(
    ({ stream: [$($stream:tt)*], kvs_ident: $kvs_ident:ident }) => {
        properties!(@ expect_key {
            stream: [$($stream)*],
            tokens: [],
            kvs_ident: $kvs_ident
        });
    };

    // We're finished parsing
    (@ expect_key {
        stream: [],
        tokens: [$($tokens:tt)*],
        kvs_ident: $kvs_ident:ident
    }) => {
        let $kvs_ident: &[(&str, $crate::kv::value::Value)] = &[$($tokens)*];
    };

    // Munch a key as an identifier from the token stream
    (@ expect_key {
        stream: [$key:ident $($stream:tt)*],
        tokens: [$($tokens:tt)*],
        kvs_ident: $kvs_ident:ident
    }) => {
        properties!(@ expect_separator {
            stream: [$($stream)*],
            tokens: [$($tokens)*],
            key: $key,
            kvs_ident: $kvs_ident
        });
    };

    // Munch a `=` from the token stream
    (@ expect_separator {
        stream: [= $($stream:tt)*],
        tokens: [$($tokens:tt)*],
        key: $key:ident,
        kvs_ident: $kvs_ident:ident
    }) => {
        properties!(@ expect_value {
            stream: [$($stream)*],
            tokens: [$($tokens)*],
            key: $key,
            kvs_ident: $kvs_ident
        });
    };
    // Munch a `:` from the token stream
    (@ expect_separator {
        stream: [: $($stream:tt)*],
        tokens: [$($tokens:tt)*],
        key: $key:ident,
        kvs_ident: $kvs_ident:ident
    }) => {
        properties!(@ expect_value {
            stream: [$($stream)*],
            tokens: [$($tokens)*],
            key: $key,
            kvs_ident: $kvs_ident
        });
    };
    // Munch a trailing comma from the token stream
    // The value is the key identifier as an expression
    (@ expect_separator {
        stream: [, $($stream:tt)*],
        tokens: [$($tokens:tt)*],
        key: $key:ident,
        kvs_ident: $kvs_ident:ident
    }) => {
        properties!(@ with_adapter {
            stream: [$($stream)*],
            tokens: [$($tokens)*],
            key: $key,
            value: $key,
            kvs_ident: $kvs_ident
        });
    };
    // We've reached the end of the token stream
    // The value is the key identifier as an expression
    (@ expect_separator {
        stream: [],
        tokens: [$($tokens:tt)*],
        key: $key:ident,
        kvs_ident: $kvs_ident:ident
    }) => {
        properties!(@ with_adapter {
            stream: [],
            tokens: [$($tokens)*],
            key: $key,
            value: $key,
            kvs_ident: $kvs_ident
        });
    };

    // Munch a value and trailing comma from the token stream
    (@ expect_value {
        stream: [$value:expr , $($stream:tt)*],
        tokens: [$($tokens:tt)*],
        key: $key:ident,
        kvs_ident: $kvs_ident:ident
    }) => {
        properties!(@ with_value {
            stream: [$($stream)*],
            tokens: [$($tokens)*],
            key: $key,
            value: $value,
            kvs_ident: $kvs_ident
        });
    };
    // Munch a value from the end of the token stream
    (@ expect_value {
        stream: [$value:expr],
        tokens: [$($tokens:tt)*],
        key: $key:ident,
        kvs_ident: $kvs_ident:ident
    }) => {
        properties!(@ with_value {
            stream: [],
            tokens: [$($tokens)*],
            key: $key,
            value: $value,
            kvs_ident: $kvs_ident
        });
    };

    // Use the value with the given adapter function
    // Borrow the keys in place and apply the adapter
    // A key value pair is pushed onto `tokens`
    (@ with_value {
        stream: [$($stream:tt)*],
        tokens: [$($tokens:tt)*],
        key: $key:ident,
        value: $value:expr,
        kvs_ident: $kvs_ident:ident
    }) => {
        let $key = &$value;

        properties!(@ expect_key {
            stream: [$($stream)*],
            tokens: [
                $($tokens)*
                (stringify!($key), $crate::kv::value::ToValue::to_value(&$key)),
            ],
            kvs_ident: $kvs_ident
        });
    };
);
