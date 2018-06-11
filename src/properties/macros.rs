#[macro_export]
macro_rules! properties(
    // Do nothing
    () => {};
    // Parse tokens between braces
    ({ $($stream:tt)* }) => {{
        __properties_internal!(@ initial {
            stream: [$($stream)*],
            properties: properties
        });
    }};
);

#[macro_export]
#[doc(hidden)]
macro_rules! __properties_internal(
    (@ initial { stream: [$($stream:tt)*], kvs_ident: $kvs_ident:ident }) => {
        __properties_internal!(@ expect_adapter {
            stream: [$($stream)*],
            tokens: [],
            kvs_ident: $kvs_ident
        });
    };

    // We're finished parsing
    // Create the set of key value pairs and 
    (@ expect_adapter {
        stream: [],
        tokens: [$($tokens:tt)*],
        kvs_ident: $kvs_ident:ident
    }) => {
        let kvs: &[(&str, &dyn $crate::properties::ToValue)] = &[$($tokens)*];
        let $kvs_ident = $crate::properties::RawKeyValues(&kvs);
    };
    // Munch a key identifier from the token stream
    (@ expect_adapter {
        stream: [$key:ident $($stream:tt)*],
        tokens: [$($tokens:tt)*],
        kvs_ident: $kvs_ident:ident
    }) => {
        __properties_internal!(@ expect_value {
            stream: [$($stream)*],
            tokens: [$($tokens)*],
            adapter: {
                kind: default
            },
            key: $key,
            kvs_ident: $kvs_ident
        });
    };
    // Munch an attribute from the token stream
    (@ expect_adapter {
        stream: [#[log($adapter:ident)] $($stream:tt)*],
        tokens: [$($tokens:tt)*],
        kvs_ident: $kvs_ident:ident
    }) => {
        __properties_internal!(@ expect_key {
            stream: [$($stream)*],
            tokens: [$($tokens)*],
            adapter: {
                kind: $adapter
            },
            kvs_ident: $kvs_ident
        });
    };
    // Munch an attribute from the token stream
    (@ expect_adapter {
        stream: [#[log($adapter_kind:ident = $adapter_state:expr)] $($stream:tt)*],
        tokens: [$($tokens:tt)*],
        kvs_ident: $kvs_ident:ident
    }) => {
        __properties_internal!(@ expect_key {
            stream: [$($stream)*],
            tokens: [$($tokens)*],
            adapter: {
                kind: $adapter_kind,
                state: $adapter_state
            },
            kvs_ident: $kvs_ident
        });
    };

    // Munch a key as an identifier from the token stream
    (@ expect_key {
        stream: [$key:ident $($stream:tt)*],
        tokens: [$($tokens:tt)*],
        adapter: { $($adapter:tt)* },
        kvs_ident: $kvs_ident:ident
    }) => {
        __properties_internal!(@ expect_value {
            stream: [$($stream)*],
            tokens: [$($tokens)*],
            adapter: { $($adapter)* },
            key: $key,
            kvs_ident: $kvs_ident
        });
    };

    // Munch a value and trailing comma from the token stream
    (@ expect_value {
        stream: [= $value:expr , $($stream:tt)*],
        tokens: [$($tokens:tt)*],
        adapter: { $($adapter:tt)* },
        key: $key:ident,
        kvs_ident: $kvs_ident:ident
    }) => {
        __properties_internal!(@ with_adapter {
            stream: [$($stream)*],
            tokens: [$($tokens)*],
            adapter: { $($adapter)* },
            key: $key,
            value: $value,
            kvs_ident: $kvs_ident
        });
    };
    // Munch a value from the end of the token stream
    (@ expect_value {
        stream: [= $value:expr],
        tokens: [$($tokens:tt)*],
        adapter: { $($adapter:tt)* },
        key: $key:ident,
        kvs_ident: $kvs_ident:ident
    }) => {
        __properties_internal!(@ with_adapter {
            stream: [],
            tokens: [$($tokens)*],
            adapter: { $($adapter)* },
            key: $key,
            value: $value,
            kvs_ident: $kvs_ident
        });
    };
    // Munch a trailing comma from the token stream
    // The value is the key identifier as an expression
    (@ expect_value {
        stream: [, $($stream:tt)*],
        tokens: [$($tokens:tt)*],
        adapter: { $($adapter:tt)* },
        key: $key:ident,
        kvs_ident: $kvs_ident:ident
    }) => {
        __properties_internal!(@ with_adapter {
            stream: [$($stream)*],
            tokens: [$($tokens)*],
            adapter: { $($adapter)* },
            key: $key,
            value: $key,
            kvs_ident: $kvs_ident
        });
    };
    // We've reached the end of the token stream
    // The value is the key identifier as an expression
    (@ expect_value {
        stream: [],
        tokens: [$($tokens:tt)*],
        adapter: { $($adapter:tt)* },
        key: $key:ident,
        kvs_ident: $kvs_ident:ident
    }) => {
        __properties_internal!(@ with_adapter {
            stream: [],
            tokens: [$($tokens)*],
            adapter: { $($adapter)* },
            key: $key,
            value: $key,
            kvs_ident: $kvs_ident
        });
    };

    // Use the adapter and replace with the default (no-op)
    // The adapter is a function like `T -> impl ToValue`
    (@ with_adapter {
        stream: [$($stream:tt)*],
        tokens: [$($tokens:tt)*],
        adapter: {
            kind: $adapter_kind:ident
        },
        key: $key:ident,
        value: $value:expr,
        kvs_ident: $kvs_ident:ident
    }) => {
        __properties_internal!(@ with_value {
            stream: [$($stream)*],
            tokens: [$($tokens)*],
            adapter_fn: $crate::properties::adapter::map::$adapter_kind,
            key: $key,
            value: $value,
            kvs_ident: $kvs_ident
        });
    };
    // Use the adapter and replace with the default (no-op)
    // The adapter is a function like `(T, F: impl Fn(&T) -> fmt::Result) -> impl ToValue`
    (@ with_adapter {
        stream: [$($stream:tt)*],
        tokens: [$($tokens:tt)*],
        adapter: {
            kind: $adapter_kind:ident,
            state: $adapter_state:expr
        },
        key: $key:ident,
        value: $value:expr,
        kvs_ident: $kvs_ident:ident
    }) => {
        __properties_internal!(@ with_value {
            stream: [$($stream)*],
            tokens: [$($tokens)*],
            adapter_fn: |value| {
                $crate::properties::adapter::map_with::$adapter_kind(value, $adapter_state)
            },
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
        adapter_fn: $adapter_fn:expr,
        key: $key:ident,
        value: $value:expr,
        kvs_ident: $kvs_ident:ident
    }) => {
        let $key = &$value;
        let $key = $adapter_fn($key);

        __properties_internal!(@ expect_adapter {
            stream: [$($stream)*],
            tokens: [
                $($tokens)*
                (stringify!($key), &$key),
            ],
            kvs_ident: $kvs_ident
        });
    };
);
