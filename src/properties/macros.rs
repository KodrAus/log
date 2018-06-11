/*!
This example demonstrates a potential macro for capturing log properties.

The macro uses a syntax that's _similar_ to struct literals. The idea is to support
extensions to the way properties are captured using attributes.

There's a bit of a misalignment between how formatting is communicated in the log
message and the contextual properties, but they are a bit different. Args are slurped 
up into the message using the formatting API whereas properties are exposed as data.

Attributes use the following syntax:

- `#[log(adapter)]` where `adapter` is a free function in `adapter::map` that takes a
generic value `&T` as an argument and returns `impl ToValue`.
- `#[log(adapter = state)]` where `adapter` is a free function in `adapter::map_with` that 
takes a generic value `&T` and `state` `S` and returns `impl ToValue`.

There are a few root adapters:

- `debug`: formats the property value using its `Debug` implementation
- `display`: formats the property using its `Display` implementation

There are a few adapters that take additional state:

- `fmt`: takes a function that's compatible with one of the `std::fmt` traits and uses
it to format the property value
- `with`: takes some function that maps a generic value `&T` to some `impl ToValue`.
This is an integration point for arbitrary formatters.

A downside of using attributes is thst one might expect standard Rust macros to work
in the same context, which they currently won't. A proc-macro based solution might be
a bit more robust and make it possible to treat the `#[log]` attributes as any other.
*/

#[macro_export]
macro_rules! properties(
    // Do nothing
    () => {};
    // Parse tokens between braces
    ({ $($stream:tt)* }) => {{
        __properties_internal!(@ expect_adapter {
            stream: [$($stream)*]
        });
    }};
);

#[macro_export]
#[doc(hidden)]
macro_rules! __properties_internal(
    // We're finished parsing
    (@ expect_adapter { stream: [] }) => { };
    // Munch a key identifier from the token stream
    (@ expect_adapter {
        stream: [$key:ident $($stream:tt)*]
    }) => {
        __properties_internal!(@ expect_value {
            stream: [$($stream)*],
            adapter: {
                kind: default
            },
            key: $key
        })
    };
    // Munch an attribute from the token stream
    (@ expect_adapter {
        stream: [#[log($adapter:ident)] $($stream:tt)*]
    }) => {
        __properties_internal!(@ expect_key {
            stream: [$($stream)*],
            adapter: {
                kind: $adapter
            }
        })
    };
    // Munch an attribute from the token stream
    (@ expect_adapter {
        stream: [#[log($adapter_kind:ident = $adapter_state:expr)] $($stream:tt)*]
    }) => {
        __properties_internal!(@ expect_key {
            stream: [$($stream)*],
            adapter: {
                kind: $adapter_kind,
                state: $adapter_state
            }
        })
    };

    // Munch a key as an identifier from the token stream
    (@ expect_key {
        stream: [$key:ident $($stream:tt)*],
        adapter: { $($adapter:tt)* }
    }) => {
        __properties_internal!(@ expect_value {
            stream: [$($stream)*],
            adapter: { $($adapter)* },
            key: $key
        })
    };

    // Munch a value and trailing comma from the token stream
    (@ expect_value {
        stream: [: $value:expr , $($stream:tt)*],
        adapter: { $($adapter:tt)* },
        key: $key:ident
    }) => {
        __properties_internal!(@ with_adapter {
            stream: [$($stream)*],
            adapter: { $($adapter)* },
            key: $key,
            value: $value
        })
    };
    // Munch a trailing comma from the token stream
    // The value is the key identifier as an expression
    (@ expect_value {
        stream: [, $($stream:tt)*],
        adapter: { $($adapter:tt)* },
        key: $key:ident
    }) => {
        __properties_internal!(@ with_adapter {
            stream: [$($stream)*],
            adapter: { $($adapter)* },
            key: $key,
            value: $key
        })
    };
    // Munch a value from the end of the token stream
    (@ expect_value {
        stream: [: $value:expr],
        adapter: { $($adapter:tt)* },
        key: $key:ident
    }) => {
        __properties_internal!(@ with_adapter {
            stream: [],
            adapter: { $($adapter)* },
            key: $key,
            value: $value
        })
    };
    // We've reached the end of the token stream
    // The value is the key identifier as an expression
    (@ expect_value {
        stream: [],
        adapter: { $($adapter:tt)* },
        key: $key:ident
    }) => {
        __properties_internal!(@ with_adapter {
            stream: [],
            adapter: { $($adapter)* },
            key: $key,
            value: $key
        })
    };

    // Use the adapter and replace with the default (no-op)
    // The adapter is a function like `T -> impl ToValue`
    (@ with_adapter {
        stream: [$($stream:tt)*],
        adapter: {
            kind: $adapter_kind:ident
        },
        key: $key:ident,
        value: $value:expr
    }) => {
        __properties_internal!(@ with_value {
            stream: [$($stream)*],
            adapter_fn: $crate::adapter::map::$adapter_kind,
            key: $key,
            value: $value
        })
    };
    // Use the adapter and replace with the default (no-op)
    // The adapter is a function like `(T, F: impl Fn(&T) -> fmt::Result) -> impl ToValue`
    (@ with_adapter {
        stream: [$($stream:tt)*],
        adapter: {
            kind: $adapter_kind:ident,
            state: $adapter_state:expr
        },
        key: $key:ident,
        value: $value:expr
    }) => {
        __properties_internal!(@ with_value {
            stream: [$($stream)*],
            adapter_fn: |value| {
                $crate::adapter::map_with::$adapter_kind(value, $adapter_state)
            },
            key: $key,
            value: $value
        })
    };

    // Use the value with no adapter
    // In this example we just print it
    (@ with_value {
        stream: [$($stream:tt)*],
        adapter_fn: $adapter_fn:expr,
        key: $key:ident,
        value: $value:expr
    }) => {
        let value = &$value;
        let adapter = $adapter_fn(value);

        // TODO: use `adapter.to_value()`

        __properties_internal!(@ expect_adapter {
            stream: [$($stream)*]
        })
    };
);
