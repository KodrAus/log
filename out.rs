#![feature(prelude_import)]
#![no_std]
// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A lightweight logging facade.
//!
//! The `log` crate provides a single logging API that abstracts over the
//! actual logging implementation. Libraries can use the logging API provided
//! by this crate, and the consumer of those libraries can choose the logging
//! implementation that is most suitable for its use case.
//!
//! If no logging implementation is selected, the facade falls back to a "noop"
//! implementation that ignores all log messages. The overhead in this case
//! is very small - just an integer load, comparison and jump.
//!
//! A log request consists of a _target_, a _level_, and a _body_. A target is a
//! string which defaults to the module path of the location of the log request,
//! though that default may be overridden. Logger implementations typically use
//! the target to filter requests based on some user configuration.
//!
//! # Use
//!
//! The basic use of the log crate is through the five logging macros: [`error!`],
//! [`warn!`], [`info!`], [`debug!`] and [`trace!`]
//! where `error!` represents the highest-priority log level, and `trace!` the lowest.
//!
//! Each of these macros accept format strings similarly to [`println!`].
//!
//!
//! [`error!`]: ./macro.error.html
//! [`warn!`]: ./macro.warn.html
//! [`info!`]: ./macro.info.html
//! [`debug!`]: ./macro.debug.html
//! [`trace!`]: ./macro.trace.html
//! [`println!`]: https://doc.rust-lang.org/stable/std/macro.println.html
//!
//! ## In libraries
//!
//! Libraries should link only to the `log` crate, and use the provided
//! macros to log whatever information will be useful to downstream consumers.
//!
//! ### Examples
//!
//! ```rust
//! # #![allow(unstable)]
//! #[macro_use]
//! extern crate log;
//!
//! # #[derive(Debug)] pub struct Yak(String);
//! # impl Yak { fn shave(&self, _: u32) {} }
//! # fn find_a_razor() -> Result<u32, u32> { Ok(1) }
//! pub fn shave_the_yak(yak: &Yak) {
//!     info!(target: "yak_events", "Commencing yak shaving for {:?}", yak);
//!
//!     loop {
//!         match find_a_razor() {
//!             Ok(razor) => {
//!                 info!("Razor located: {}", razor);
//!                 yak.shave(razor);
//!                 break;
//!             }
//!             Err(err) => {
//!                 warn!("Unable to locate a razor: {}, retrying", err);
//!             }
//!         }
//!     }
//! }
//! # fn main() {}
//! ```
//!
//! ## In executables
//!
//! Executables should choose a logging implementation and initialize it early in the
//! runtime of the program. Logging implementations will typically include a
//! function to do this. Any log messages generated before
//! the implementation is initialized will be ignored.
//!
//! The executable itself may use the `log` crate to log as well.
//!
//! ### Warning
//!
//! The logging system may only be initialized once.
//!
//! # Available logging implementations
//!
//! In order to produce log output executables have to use
//! a logger implementation compatible with the facade.
//! There are many available implementations to choose from,
//! here are some of the most popular ones:
//!
//! * Simple minimal loggers:
//!     * [env_logger]
//!     * [simple_logger]
//!     * [simplelog]
//!     * [pretty_env_logger]
//!     * [stderrlog]
//!     * [flexi_logger]
//! * Complex configurable frameworks:
//!     * [log4rs]
//!     * [fern]
//! * Adaptors for other facilities:
//!     * [syslog]
//!     * [slog-stdlog]
//!
//! # Implementing a Logger
//!
//! Loggers implement the [`Log`] trait. Here's a very basic example that simply
//! logs all messages at the [`Error`][level_link], [`Warn`][level_link] or
//! [`Info`][level_link] levels to stdout:
//!
//! ```rust
//! extern crate log;
//!
//! use log::{Record, Level, Metadata};
//!
//! struct SimpleLogger;
//!
//! impl log::Log for SimpleLogger {
//!     fn enabled(&self, metadata: &Metadata) -> bool {
//!         metadata.level() <= Level::Info
//!     }
//!
//!     fn log(&self, record: &Record) {
//!         if self.enabled(record.metadata()) {
//!             println!("{} - {}", record.level(), record.args());
//!         }
//!     }
//!
//!     fn flush(&self) {}
//! }
//!
//! # fn main() {}
//! ```
//!
//! Loggers are installed by calling the [`set_logger`] function. The maximum
//! log level also needs to be adjusted via the [`set_max_level`] function. The
//! logging facade uses this as an optimization to improve performance of log
//! messages at levels that are disabled. It's important to set it, as it
//! defaults to [`Off`][filter_link], so no log messages will ever be captured!
//! In the case of our example logger, we'll want to set the maximum log level
//! to [`Info`][filter_link], since we ignore any [`Debug`][level_link] or
//! [`Trace`][level_link] level log messages. A logging implementation should
//! provide a function that wraps a call to [`set_logger`] and
//! [`set_max_level`], handling initialization of the logger:
//!
//! ```rust
//! # extern crate log;
//! # use log::{Level, Metadata};
//! # struct SimpleLogger;
//! # impl log::Log for SimpleLogger {
//! #   fn enabled(&self, _: &Metadata) -> bool { false }
//! #   fn log(&self, _: &log::Record) {}
//! #   fn flush(&self) {}
//! # }
//! # fn main() {}
//! use log::{SetLoggerError, LevelFilter};
//!
//! static LOGGER: SimpleLogger = SimpleLogger;
//!
//! pub fn init() -> Result<(), SetLoggerError> {
//!     log::set_logger(&LOGGER)
//!         .map(|()| log::set_max_level(LevelFilter::Info))
//! }
//! ```
//!
//! Implementations that adjust their configurations at runtime should take care
//! to adjust the maximum log level as well.
//!
//! # Use with `std`
//!
//! `set_logger` requires you to provide a `&'static Log`, which can be hard to
//! obtain if your logger depends on some runtime configuration. The
//! `set_boxed_logger` function is available with the `std` Cargo feature. It is
//! identical to `set_logger` except that it takes a `Box<Log>` rather than a
//! `&'static Log`:
//!
//! ```rust
//! # extern crate log;
//! # use log::{Level, LevelFilter, Log, SetLoggerError, Metadata};
//! # struct SimpleLogger;
//! # impl log::Log for SimpleLogger {
//! #   fn enabled(&self, _: &Metadata) -> bool { false }
//! #   fn log(&self, _: &log::Record) {}
//! #   fn flush(&self) {}
//! # }
//! # fn main() {}
//! # #[cfg(feature = "std")]
//! pub fn init() -> Result<(), SetLoggerError> {
//!     log::set_boxed_logger(Box::new(SimpleLogger))
//!         .map(|()| log::set_max_level(LevelFilter::Info))
//! }
//! ```
//!
//! # Compile time filters
//!
//! Log levels can be statically disabled at compile time via Cargo features. Log invocations at
//! disabled levels will be skipped and will not even be present in the resulting binary unless the
//! log level is specified dynamically. This level is configured separately for release and debug
//! builds. The features are:
//!
//! * `max_level_off`
//! * `max_level_error`
//! * `max_level_warn`
//! * `max_level_info`
//! * `max_level_debug`
//! * `max_level_trace`
//! * `release_max_level_off`
//! * `release_max_level_error`
//! * `release_max_level_warn`
//! * `release_max_level_info`
//! * `release_max_level_debug`
//! * `release_max_level_trace`
//!
//! These features control the value of the `STATIC_MAX_LEVEL` constant. The logging macros check
//! this value before logging a message. By default, no levels are disabled.
//!
//! For example, a crate can disable trace level logs in debug builds and trace, info, and warn
//! level logs in release builds with the following configuration:
//!
//! ```toml
//! [dependencies]
//! log = { version = "0.4", features = ["max_level_debug", "release_max_level_warn"] }
//! ```
//!
//! # Version compatibility
//!
//! The 0.3 and 0.4 versions of the `log` crate are almost entirely compatible. Log messages
//! made using `log` 0.3 will forward transparently to a logger implementation using `log` 0.4. Log
//! messages made using `log` 0.4 will forward to a logger implementation using `log` 0.3, but the
//! module path and file name information associated with the message will unfortunately be lost.
//!
//! [`Log`]: trait.Log.html
//! [level_link]: enum.Level.html
//! [filter_link]: enum.LevelFilter.html
//! [`set_logger`]: fn.set_logger.html
//! [`set_max_level`]: fn.set_max_level.html
//! [`try_set_logger_raw`]: fn.try_set_logger_raw.html
//! [`shutdown_logger_raw`]: fn.shutdown_logger_raw.html
//! [env_logger]: https://docs.rs/env_logger/*/env_logger/
//! [simple_logger]: https://github.com/borntyping/rust-simple_logger
//! [simplelog]: https://github.com/drakulix/simplelog.rs
//! [pretty_env_logger]: https://docs.rs/pretty_env_logger/*/pretty_env_logger/
//! [stderrlog]: https://docs.rs/stderrlog/*/stderrlog/
//! [flexi_logger]: https://docs.rs/flexi_logger/*/flexi_logger/
//! [syslog]: https://docs.rs/syslog/*/syslog/
//! [slog-stdlog]: https://docs.rs/slog-stdlog/*/slog_stdlog/
//! [log4rs]: https://docs.rs/log4rs/*/log4rs/
//! [fern]: https://docs.rs/fern/*/fern/

#![doc(
    html_logo_url = "https://www.rust-lang.org/logos/rust-logo-128x128-blk-v2.png",
    html_favicon_url = "https://www.rust-lang.org/favicon.ico",
    html_root_url = "https://docs.rs/log/0.4.1"
)]
#![warn(missing_docs)]
#![deny(missing_debug_implementations)]
#[prelude_import]
use std::prelude::v1::*;
#[macro_use]
extern crate std;
// When compiled for the rustc compiler itself we want to make sure that this is
// an unstable crate

#[cfg(any(feature = "std", feature = "alloc"))]
extern crate erased_serde;
#[cfg(feature = "serde")]
extern crate serde;

#[macro_use]
extern crate cfg_if;

use std::cmp;
#[cfg(feature = "std")]
use std::error;
use std::fmt;
use std::mem;
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT};

#[macro_use]
mod macros {

    // The LOGGER static holds a pointer to the global logger. It is protected by
    // the STATE static which determines whether LOGGER has been initialized yet.

    // There are three different states that we care about: the logger's
    // uninitialized, the logger's initializing (set_logger's been called but
    // LOGGER hasn't actually been set yet), or the logger's active.

    // This way these line up with the discriminants for LevelFilter below

    // Reimplemented here because std::ascii is not available in libcore

    // Deriving generates terrible impls of these traits

    // Just used as a dummy initial value for LOGGER

    // The Error trait is not available in libcore

    // The Error trait is not available in libcore

    /// The standard logging macro.
    ///
    /// This macro will generically log with the specified `Level` and `format!`
    /// based argument list.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate log;
    /// use log::Level;
    ///
    /// # fn main() {
    /// let data = (42, "Forty-two");
    /// let private_data = "private";
    ///
    /// log!(Level::Error, "Received errors: {}, {}", data.0, data.1);
    /// log!(target: "app_events", Level::Warn, "App warning: {}, {}, {}",
    ///     data.0, data.1, private_data);
    /// # }
    /// ```
    #[macro_export]
    macro_rules! log((
                     target : $ target : expr , $ lvl : expr , $ ( $ arg : tt
                     ) + ) => (
                     {
                     let lvl = $ lvl ; if lvl <= $ crate :: STATIC_MAX_LEVEL
                     && lvl <= $ crate :: max_level (  ) {
                     $ crate :: Log :: log (
                     $ crate :: logger (  ) , & $ crate :: RecordBuilder ::
                     new (  ) . args ( format_args ! ( $ ( $ arg ) + ) ) .
                     level ( lvl ) . target ( $ target ) . module_path (
                     Some ( module_path ! (  ) ) ) . file (
                     Some ( file ! (  ) ) ) . line ( Some ( line ! (  ) ) ) .
                     build (  ) ) } } ) ; ( $ lvl : expr , $ ( $ arg : tt ) +
                     ) => (
                     log ! (
                     target : module_path ! (  ) , $ lvl , $ ( $ arg ) + ) ));
    /// Logs a message at the error level.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate log;
    /// # fn main() {
    /// let (err_info, port) = ("No connection", 22);
    ///
    /// error!("Error: {} on port {}", err_info, port);
    /// error!(target: "app_events", "App Error: {}, Port: {}", err_info, 22);
    /// # }
    /// ```
    #[macro_export]
    macro_rules! error(( target : $ target : expr , $ ( $ arg : tt ) * ) => (
                       log ! (
                       target : $ target , $ crate :: Level :: Error , $ (
                       $ arg ) * ) ; ) ; ( $ ( $ arg : tt ) * ) => (
                       log ! ( $ crate :: Level :: Error , $ ( $ arg ) * ) ;
                       ));
    /// Logs a message at the warn level.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate log;
    /// # fn main() {
    /// let warn_description = "Invalid Input";
    ///
    /// warn!("Warning! {}!", warn_description);
    /// warn!(target: "input_events", "App received warning: {}", warn_description);
    /// # }
    /// ```
    #[macro_export]
    macro_rules! warn(( target : $ target : expr , $ ( $ arg : tt ) * ) => (
                      log ! (
                      target : $ target , $ crate :: Level :: Warn , $ ( $ arg
                      ) * ) ; ) ; ( $ ( $ arg : tt ) * ) => (
                      log ! ( $ crate :: Level :: Warn , $ ( $ arg ) * ) ; ));
    /// Logs a message at the info level.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate log;
    /// # fn main() {
    /// # struct Connection { port: u32, speed: f32 }
    /// let conn_info = Connection { port: 40, speed: 3.20 };
    ///
    /// info!("Connected to port {} at {} Mb/s", conn_info.port, conn_info.speed);
    /// info!(target: "connection_events", "Successfull connection, port: {}, speed: {}",
    ///       conn_info.port, conn_info.speed);
    /// # }
    /// ```
    #[macro_export]
    macro_rules! info(( target : $ target : expr , $ ( $ arg : tt ) * ) => (
                      log ! (
                      target : $ target , $ crate :: Level :: Info , $ ( $ arg
                      ) * ) ; ) ; ( $ ( $ arg : tt ) * ) => (
                      log ! ( $ crate :: Level :: Info , $ ( $ arg ) * ) ; ));
    /// Logs a message at the debug level.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate log;
    /// # fn main() {
    /// # struct Position { x: f32, y: f32 }
    /// let pos = Position { x: 3.234, y: -1.223 };
    ///
    /// debug!("New position: x: {}, y: {}", pos.x, pos.y);
    /// debug!(target: "app_events", "New position: x: {}, y: {}", pos.x, pos.y);
    /// # }
    /// ```
    #[macro_export]
    macro_rules! debug(( target : $ target : expr , $ ( $ arg : tt ) * ) => (
                       log ! (
                       target : $ target , $ crate :: Level :: Debug , $ (
                       $ arg ) * ) ; ) ; ( $ ( $ arg : tt ) * ) => (
                       log ! ( $ crate :: Level :: Debug , $ ( $ arg ) * ) ;
                       ));
    /// Logs a message at the trace level.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate log;
    /// # fn main() {
    /// # struct Position { x: f32, y: f32 }
    /// let pos = Position { x: 3.234, y: -1.223 };
    ///
    /// trace!("Position is: x: {}, y: {}", pos.x, pos.y);
    /// trace!(target: "app_events", "x is {} and y is {}",
    ///        if pos.x >= 0.0 { "positive" } else { "negative" },
    ///        if pos.y >= 0.0 { "positive" } else { "negative" });
    /// # }
    /// ```
    #[macro_export]
    macro_rules! trace(( target : $ target : expr , $ ( $ arg : tt ) * ) => (
                       log ! (
                       target : $ target , $ crate :: Level :: Trace , $ (
                       $ arg ) * ) ; ) ; ( $ ( $ arg : tt ) * ) => (
                       log ! ( $ crate :: Level :: Trace , $ ( $ arg ) * ) ;
                       ));
    /// Determines if a message logged at the specified level in that module will
    /// be logged.
    ///
    /// This can be used to avoid expensive computation of log message arguments if
    /// the message would be ignored anyway.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # #[macro_use]
    /// # extern crate log;
    /// use log::Level::Debug;
    ///
    /// # fn foo() {
    /// if log_enabled!(Debug) {
    ///     let data = expensive_call();
    ///     debug!("expensive debug data: {} {}", data.x, data.y);
    /// }
    /// if log_enabled!(target: "Global", Debug) {
    ///    let data = expensive_call();
    ///    debug!(target: "Global", "expensive debug data: {} {}", data.x, data.y);
    /// }
    /// # }
    /// # struct Data { x: u32, y: u32 }
    /// # fn expensive_call() -> Data { Data { x: 0, y: 0 } }
    /// # fn main() {}
    /// ```
    #[macro_export]
    macro_rules! log_enabled(( target : $ target : expr , $ lvl : expr ) => (
                             {
                             let lvl = $ lvl ; lvl <= $ crate ::
                             STATIC_MAX_LEVEL && lvl <= $ crate :: max_level (
                              ) && $ crate :: Log :: enabled (
                             $ crate :: logger (  ) , & $ crate ::
                             MetadataBuilder :: new (  ) . level ( lvl ) .
                             target ( $ target ) . build (  ) , ) } ) ; (
                             $ lvl : expr ) => (
                             log_enabled ! (
                             target : module_path ! (  ) , $ lvl ) ));
}
#[cfg(feature = "serde")]
mod serde_support {
    use serde::de::{
        Deserialize, DeserializeSeed, Deserializer, EnumAccess, Error, VariantAccess, Visitor,
    };
    use serde::ser::{Serialize, Serializer};
    use std::fmt;
    use std::str::FromStr;
    use {Level, LevelFilter, LOG_LEVEL_NAMES};
    impl Serialize for Level {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match *self {
                Level::Error => serializer.serialize_unit_variant("Level", 0, "ERROR"),
                Level::Warn => serializer.serialize_unit_variant("Level", 1, "WARN"),
                Level::Info => serializer.serialize_unit_variant("Level", 2, "INFO"),
                Level::Debug => serializer.serialize_unit_variant("Level", 3, "DEBUG"),
                Level::Trace => serializer.serialize_unit_variant("Level", 4, "TRACE"),
            }
        }
    }
    impl<'de> Deserialize<'de> for Level {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct LevelIdentifier;
            impl<'de> Visitor<'de> for LevelIdentifier {
                type Value = Level;
                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("log level")
                }
                fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
                where
                    E: Error,
                {
                    FromStr::from_str(s)
                        .map_err(|_| Error::unknown_variant(s, &LOG_LEVEL_NAMES[1..]))
                }
            }
            impl<'de> DeserializeSeed<'de> for LevelIdentifier {
                type Value = Level;
                fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    deserializer.deserialize_identifier(LevelIdentifier)
                }
            }
            struct LevelEnum;
            impl<'de> Visitor<'de> for LevelEnum {
                type Value = Level;
                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("log level")
                }
                fn visit_enum<A>(self, value: A) -> Result<Self::Value, A::Error>
                where
                    A: EnumAccess<'de>,
                {
                    let (level, variant) = value.variant_seed(LevelIdentifier)?;
                    variant.unit_variant()?;
                    Ok(level)
                }
            }
            deserializer.deserialize_enum("Level", &LOG_LEVEL_NAMES[1..], LevelEnum)
        }
    }
    impl Serialize for LevelFilter {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match *self {
                LevelFilter::Off => serializer.serialize_unit_variant("LevelFilter", 0, "OFF"),
                LevelFilter::Error => serializer.serialize_unit_variant("LevelFilter", 1, "ERROR"),
                LevelFilter::Warn => serializer.serialize_unit_variant("LevelFilter", 2, "WARN"),
                LevelFilter::Info => serializer.serialize_unit_variant("LevelFilter", 3, "INFO"),
                LevelFilter::Debug => serializer.serialize_unit_variant("LevelFilter", 4, "DEBUG"),
                LevelFilter::Trace => serializer.serialize_unit_variant("LevelFilter", 5, "TRACE"),
            }
        }
    }
    impl<'de> Deserialize<'de> for LevelFilter {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct LevelFilterIdentifier;
            impl<'de> Visitor<'de> for LevelFilterIdentifier {
                type Value = LevelFilter;
                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("log level filter")
                }
                fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
                where
                    E: Error,
                {
                    FromStr::from_str(s).map_err(|_| Error::unknown_variant(s, &LOG_LEVEL_NAMES))
                }
            }
            impl<'de> DeserializeSeed<'de> for LevelFilterIdentifier {
                type Value = LevelFilter;
                fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    deserializer.deserialize_identifier(LevelFilterIdentifier)
                }
            }
            struct LevelFilterEnum;
            impl<'de> Visitor<'de> for LevelFilterEnum {
                type Value = LevelFilter;
                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("log level filter")
                }
                fn visit_enum<A>(self, value: A) -> Result<Self::Value, A::Error>
                where
                    A: EnumAccess<'de>,
                {
                    let (level_filter, variant) = value.variant_seed(LevelFilterIdentifier)?;
                    variant.unit_variant()?;
                    Ok(level_filter)
                }
            }
            deserializer.deserialize_enum("LevelFilter", &LOG_LEVEL_NAMES, LevelFilterEnum)
        }
    }
}
#[cfg(feature = "serde")]
#[macro_use]
pub mod properties {
    //! Log record properties.
    #[macro_use]
    mod macros {
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
        macro_rules! properties((  ) => {  } ; ( { $ ( $ stream : tt ) * } )
                                => {
                                {
                                __properties_internal ! (
                                @ initial {
                                stream : [ $ ( $ stream ) * ] , properties :
                                properties } ) ; } } ;);
        #[macro_export]
        #[doc(hidden)]
        macro_rules! __properties_internal((
                                           @ initial {
                                           stream : [ $ ( $ stream : tt ) * ]
                                           , properties : $ properties : ident
                                           } ) => {
                                           let $ properties = $ crate ::
                                           properties :: Properties :: empty (
                                            ) ; __properties_internal ! (
                                           @ expect_adapter {
                                           stream : [ $ ( $ stream ) * ] ,
                                           properties : $ properties } ) ; } ;
                                           (
                                           @ expect_adapter {
                                           stream : [  ] , properties : $
                                           properties : ident } ) => {  } ; (
                                           @ expect_adapter {
                                           stream : [
                                           $ key : ident $ ( $ stream : tt ) *
                                           ] , properties : $ properties :
                                           ident } ) => {
                                           __properties_internal ! (
                                           @ expect_value {
                                           stream : [ $ ( $ stream ) * ] ,
                                           adapter : { kind : default } , key
                                           : $ key , properties : $ properties
                                           } ) ; } ; (
                                           @ expect_adapter {
                                           stream : [
                                           # [ log ( $ adapter : ident ) ] $ (
                                           $ stream : tt ) * ] , properties :
                                           $ properties : ident } ) => {
                                           __properties_internal ! (
                                           @ expect_key {
                                           stream : [ $ ( $ stream ) * ] ,
                                           adapter : { kind : $ adapter } ,
                                           properties : $ properties } ) ; } ;
                                           (
                                           @ expect_adapter {
                                           stream : [
                                           # [
                                           log (
                                           $ adapter_kind : ident = $
                                           adapter_state : expr ) ] $ (
                                           $ stream : tt ) * ] , properties :
                                           $ properties : ident } ) => {
                                           __properties_internal ! (
                                           @ expect_key {
                                           stream : [ $ ( $ stream ) * ] ,
                                           adapter : {
                                           kind : $ adapter_kind , state : $
                                           adapter_state } , properties : $
                                           properties } ) ; } ; (
                                           @ expect_key {
                                           stream : [
                                           $ key : ident $ ( $ stream : tt ) *
                                           ] , adapter : {
                                           $ ( $ adapter : tt ) * } ,
                                           properties : $ properties : ident }
                                           ) => {
                                           __properties_internal ! (
                                           @ expect_value {
                                           stream : [ $ ( $ stream ) * ] ,
                                           adapter : { $ ( $ adapter ) * } ,
                                           key : $ key , properties : $
                                           properties } ) ; } ; (
                                           @ expect_value {
                                           stream : [
                                           : $ value : expr , $ (
                                           $ stream : tt ) * ] , adapter : {
                                           $ ( $ adapter : tt ) * } , key : $
                                           key : ident , properties : $
                                           properties : ident } ) => {
                                           __properties_internal ! (
                                           @ with_adapter {
                                           stream : [ $ ( $ stream ) * ] ,
                                           adapter : { $ ( $ adapter ) * } ,
                                           key : $ key , value : $ value ,
                                           properties : $ properties } ) ; } ;
                                           (
                                           @ expect_value {
                                           stream : [ , $ ( $ stream : tt ) *
                                           ] , adapter : {
                                           $ ( $ adapter : tt ) * } , key : $
                                           key : ident , properties : $
                                           properties : ident } ) => {
                                           __properties_internal ! (
                                           @ with_adapter {
                                           stream : [ $ ( $ stream ) * ] ,
                                           adapter : { $ ( $ adapter ) * } ,
                                           key : $ key , value : $ key ,
                                           properties : $ properties } ) ; } ;
                                           (
                                           @ expect_value {
                                           stream : [ : $ value : expr ] ,
                                           adapter : { $ ( $ adapter : tt ) *
                                           } , key : $ key : ident ,
                                           properties : $ properties : ident }
                                           ) => {
                                           __properties_internal ! (
                                           @ with_adapter {
                                           stream : [  ] , adapter : {
                                           $ ( $ adapter ) * } , key : $ key ,
                                           value : $ value , properties : $
                                           properties } ) ; } ; (
                                           @ expect_value {
                                           stream : [  ] , adapter : {
                                           $ ( $ adapter : tt ) * } , key : $
                                           key : ident , properties : $
                                           properties : ident } ) => {
                                           __properties_internal ! (
                                           @ with_adapter {
                                           stream : [  ] , adapter : {
                                           $ ( $ adapter ) * } , key : $ key ,
                                           value : $ key , properties : $
                                           properties } ) ; } ; (
                                           @ with_adapter {
                                           stream : [ $ ( $ stream : tt ) * ]
                                           , adapter : {
                                           kind : $ adapter_kind : ident } ,
                                           key : $ key : ident , value : $
                                           value : expr , properties : $
                                           properties : ident } ) => {
                                           __properties_internal ! (
                                           @ with_value {
                                           stream : [ $ ( $ stream ) * ] ,
                                           adapter_fn : $ crate :: properties
                                           :: adapter :: map :: $ adapter_kind
                                           , key : $ key , value : $ value ,
                                           properties : $ properties } ) ; } ;
                                           (
                                           @ with_adapter {
                                           stream : [ $ ( $ stream : tt ) * ]
                                           , adapter : {
                                           kind : $ adapter_kind : ident ,
                                           state : $ adapter_state : expr } ,
                                           key : $ key : ident , value : $
                                           value : expr , properties : $
                                           properties : ident } ) => {
                                           __properties_internal ! (
                                           @ with_value {
                                           stream : [ $ ( $ stream ) * ] ,
                                           adapter_fn : | value | {
                                           $ crate :: properties :: adapter ::
                                           map_with :: $ adapter_kind (
                                           value , $ adapter_state ) } , key :
                                           $ key , value : $ value ,
                                           properties : $ properties } ) ; } ;
                                           (
                                           @ with_value {
                                           stream : [ $ ( $ stream : tt ) * ]
                                           , adapter_fn : $ adapter_fn : expr
                                           , key : $ key : ident , value : $
                                           value : expr , properties : $
                                           properties : ident } ) => {
                                           let value = & $ value ; let adapter
                                           = $ adapter_fn ( value ) ; let kvs
                                           = $ crate :: properties ::
                                           RawKeyValue (
                                           stringify ! ( $ key ) , & adapter )
                                           ; let $ properties = $ crate ::
                                           properties :: Properties :: chained
                                           ( & kvs , & $ properties ) ;
                                           __properties_internal ! (
                                           @ expect_adapter {
                                           stream : [ $ ( $ stream ) * ] ,
                                           properties : $ properties } ) ; }
                                           ;);
    }
    mod value {
        #[cfg(feature = "erased-serde")]
        use erased_serde;
        use properties;
        use serde;
        use std::fmt;
        /// A single property value.
        ///
        /// Values implement `serde::Serialize`.
        pub struct Value<'a> {
            inner: ValueInner<'a>,
        }
        #[rustc_copy_clone_marker]
        enum ValueInner<'a> {
            Fmt(&'a dyn fmt::Debug),

            #[cfg(feature = "erased-serde")]
            Serde(&'a dyn erased_serde::Serialize),
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl<'a> ::std::clone::Clone for ValueInner<'a> {
            #[inline]
            fn clone(&self) -> ValueInner<'a> {
                {
                    let _: ::std::clone::AssertParamIsClone<&'a dyn fmt::Debug>;
                    let _: ::std::clone::AssertParamIsClone<
                        &'a dyn erased_serde::Serialize,
                    >;
                    *self
                }
            }
        }
        #[automatically_derived]
        #[allow(unused_qualifications)]
        impl<'a> ::std::marker::Copy for ValueInner<'a> {}
        impl<'a> serde::Serialize for Value<'a> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                match self.inner {
                    ValueInner::Fmt(v) => {
                        struct FmtAdapter<T>(T);
                        impl<T> fmt::Display for FmtAdapter<T>
                        where
                            T: fmt::Debug,
                        {
                            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                                self.0.fmt(f)
                            }
                        }
                        serializer.collect_str(&FmtAdapter(v))
                    }
                    #[cfg(feature = "erased-serde")]
                    ValueInner::Serde(v) => v.serialize(serializer),
                }
            }
        }
        impl<'a> Value<'a> {
            pub fn new(v: &'a (impl serde::Serialize + fmt::Debug)) -> Self {
                Value {
                    inner: {
                        #[cfg(feature = "erased-serde")]
                        {
                            ValueInner::Serde(v)
                        }
                    },
                }
            }
            pub fn fmt(v: &'a impl fmt::Debug) -> Self {
                Value {
                    inner: ValueInner::Fmt(v),
                }
            }
            #[cfg(feature = "erased-serde")]
            pub fn serde(v: &'a impl serde::Serialize) -> Self {
                Value {
                    inner: ValueInner::Serde(v),
                }
            }
        }
        impl<'a> fmt::Debug for Value<'a> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.debug_struct("Value").finish()
            }
        }
        pub trait ToValue {
            fn to_value(&self) -> Value;
        }
        impl<'a> ToValue for Value<'a> {
            fn to_value(&self) -> Value {
                Value { inner: self.inner }
            }
        }
        impl<'a, T: ?Sized> ToValue for &'a T
        where
            T: ToValue,
        {
            fn to_value(&self) -> Value {
                (*self).to_value()
            }
        }
    }
    pub mod adapter {
        pub mod map {
            use super::*;
            use properties::{ToValue, Value};
            use serde;
            use std::fmt::{Debug, Display};
            /// The default property adapter used when no `#[log]` attribute is present.
            ///
            /// If `std` is available, this will use `Serialize`.
            /// If `std` is not available, this will use `Debug`.
            pub fn default(v: impl serde::Serialize + Debug) -> impl ToValue {
                #[cfg(feature = "erased-serde")]
                {
                    serde(v)
                }
            }
            /// `#[log(serde)]` Format a property value using its `Serialize` implementation.
            ///
            /// The property value will retain its structure.
            #[cfg(feature = "erased-serde")]
            pub fn serde(v: impl serde::Serialize) -> impl ToValue {
                struct SerdeAdapter<T>(T);
                #[automatically_derived]
                #[allow(unused_qualifications)]
                impl<T: ::std::fmt::Debug> ::std::fmt::Debug for SerdeAdapter<T> {
                    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                        match *self {
                            SerdeAdapter(ref __self_0_0) => {
                                let mut debug_trait_builder = f.debug_tuple("SerdeAdapter");
                                let _ = debug_trait_builder.field(&&(*__self_0_0));
                                debug_trait_builder.finish()
                            }
                        }
                    }
                }
                impl<T> ToValue for SerdeAdapter<T>
                where
                    T: serde::Serialize,
                {
                    fn to_value(&self) -> Value {
                        Value::serde(&self.0)
                    }
                }
                SerdeAdapter(v)
            }
            /// `#[log(debug)]` Format a property value using its `Debug` implementation.
            ///
            /// The property value will be serialized as a string.
            pub fn debug(v: impl Debug) -> impl ToValue {
                map_with::fmt(v, Debug::fmt)
            }
            /// `#[log(display)]` Format a property value using its `Display` implementation.
            ///
            /// The property value will be serialized as a string.
            pub fn display(v: impl Display) -> impl ToValue {
                map_with::fmt(v, Display::fmt)
            }
        }
        pub mod map_with {
            use super::*;
            use properties::{ToValue, Value};
            use std::fmt::{Debug, Display, Formatter, Result};
            /// `#[log(fmt = expr)]` Format a property value using a specific format.
            pub fn fmt<T>(
                value: T,
                adapter: impl Fn(&T, &mut Formatter) -> Result,
            ) -> impl ToValue {
                struct FmtAdapter<T, F> {
                    value: T,
                    adapter: F,
                }
                impl<T, F> Debug for FmtAdapter<T, F>
                where
                    F: Fn(&T, &mut Formatter) -> Result,
                {
                    fn fmt(&self, f: &mut Formatter) -> Result {
                        (self.adapter)(&self.value, f)
                    }
                }
                impl<T, F> ToValue for FmtAdapter<T, F>
                where
                    F: Fn(&T, &mut Formatter) -> Result,
                {
                    fn to_value(&self) -> Value {
                        Value::fmt(self)
                    }
                }
                FmtAdapter { value, adapter }
            }
            /// `#[log(with = expr)]` Use a generic adapter.
            pub fn with<T, U>(value: T, adapter: impl Fn(T) -> U) -> U
            where
                U: ToValue,
            {
                adapter(value)
            }
        }
    }
    pub use self::value::*;
    use serde;
    use std::fmt;
    /// A serializer for key value pairs.
    pub trait Serializer {
        /// Serialize the key and value.
        fn serialize_kv(&mut self, kv: &dyn KeyValue);
    }
    /// A set of key value pairs that can be serialized.
    pub trait KeyValues {
        /// Serialize the key value pairs.
        fn serialize(&self, serializer: &mut dyn Serializer);
    }
    /// A single key value pair.
    pub trait KeyValue {
        /// Get the key.
        fn key(&self) -> &str;
        /// Get the value.
        fn value(&self) -> Value;
    }
    impl<K, V> KeyValue for (K, V)
    where
        K: AsRef<str>,
        V: ToValue,
    {
        fn key(&self) -> &str {
            self.0.as_ref()
        }
        fn value(&self) -> Value {
            self.1.to_value()
        }
    }
    impl<'a, T: ?Sized> KeyValue for &'a T
    where
        T: KeyValue,
    {
        fn key(&self) -> &str {
            (*self).key()
        }
        fn value(&self) -> Value {
            (*self).value()
        }
    }
    impl<'a, T: ?Sized, KV> KeyValues for &'a T
    where
        &'a T: IntoIterator<Item = KV>,
        KV: KeyValue,
    {
        fn serialize(&self, serializer: &mut dyn Serializer) {
            for kv in self.into_iter() {
                serializer.serialize_kv(&kv);
            }
        }
    }
    pub struct SerializeMap<T>(T);
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<T: ::std::fmt::Debug> ::std::fmt::Debug for SerializeMap<T> {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            match *self {
                SerializeMap(ref __self_0_0) => {
                    let mut debug_trait_builder = f.debug_tuple("SerializeMap");
                    let _ = debug_trait_builder.field(&&(*__self_0_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl<T> SerializeMap<T> {
        fn new(inner: T) -> Self {
            SerializeMap(inner)
        }
        fn into_inner(self) -> T {
            self.0
        }
    }
    impl<T> Serializer for SerializeMap<T>
    where
        T: serde::ser::SerializeMap,
    {
        fn serialize_kv(&mut self, kv: &dyn KeyValue) {
            let _ = serde::ser::SerializeMap::serialize_entry(&mut self.0, kv.key(), &kv.value());
        }
    }
    impl<KV> serde::Serialize for SerializeMap<KV>
    where
        KV: KeyValues,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            use serde::ser::SerializeMap as SerializeTrait;
            let mut map = SerializeMap::new(serializer.serialize_map(None)?);
            KeyValues::serialize(&self.0, &mut map);
            map.into_inner().end()
        }
    }
    pub struct SerializeSeq<T>(T);
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<T: ::std::fmt::Debug> ::std::fmt::Debug for SerializeSeq<T> {
        fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            match *self {
                SerializeSeq(ref __self_0_0) => {
                    let mut debug_trait_builder = f.debug_tuple("SerializeSeq");
                    let _ = debug_trait_builder.field(&&(*__self_0_0));
                    debug_trait_builder.finish()
                }
            }
        }
    }
    impl<T> SerializeSeq<T> {
        fn new(inner: T) -> Self {
            SerializeSeq(inner)
        }
        fn into_inner(self) -> T {
            self.0
        }
    }
    impl<T> Serializer for SerializeSeq<T>
    where
        T: serde::ser::SerializeSeq,
    {
        fn serialize_kv(&mut self, kv: &dyn KeyValue) {
            let _ =
                serde::ser::SerializeSeq::serialize_element(&mut self.0, &(kv.key(), kv.value()));
        }
    }
    impl<KV> serde::Serialize for SerializeSeq<KV>
    where
        KV: KeyValues,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            use serde::ser::SerializeSeq as SerializeTrait;
            let mut seq = SerializeSeq::new(serializer.serialize_seq(None)?);
            KeyValues::serialize(&self.0, &mut seq);
            seq.into_inner().end()
        }
    }
    struct EmptyKeyValue;
    impl KeyValues for EmptyKeyValue {
        fn serialize(&self, serializer: &mut dyn Serializer) {}
    }
    #[doc(hidden)]
    pub struct RawKeyValue<'a>(pub &'a str, pub &'a dyn ToValue);
    impl<'a> fmt::Debug for RawKeyValue<'a> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.debug_struct("RawKeyValue").finish()
        }
    }
    impl<'a> KeyValues for RawKeyValue<'a> {
        fn serialize(&self, serializer: &mut dyn Serializer) {
            serializer.serialize_kv(&(self.0, self.1))
        }
    }
    /// A chain of properties.
    pub struct Properties<'a> {
        kvs: &'a dyn KeyValues,
        parent: Option<&'a Properties<'a>>,
    }
    #[automatically_derived]
    #[allow(unused_qualifications)]
    impl<'a> ::std::clone::Clone for Properties<'a> {
        #[inline]
        fn clone(&self) -> Properties<'a> {
            match *self {
                Properties {
                    kvs: ref __self_0_0,
                    parent: ref __self_0_1,
                } => Properties {
                    kvs: ::std::clone::Clone::clone(&(*__self_0_0)),
                    parent: ::std::clone::Clone::clone(&(*__self_0_1)),
                },
            }
        }
    }
    impl<'a> Properties<'a> {
        pub fn empty() -> Self {
            Properties {
                kvs: &EmptyKeyValue,
                parent: None,
            }
        }
        pub fn root(properties: &'a dyn KeyValues) -> Self {
            Properties {
                kvs: properties,
                parent: None,
            }
        }
        pub fn chained(properties: &'a dyn KeyValues, parent: &'a Properties) -> Self {
            Properties {
                kvs: properties,
                parent: Some(parent),
            }
        }
        pub fn serialize_map(&self) -> SerializeMap<&Self> {
            SerializeMap::new(&self)
        }
        pub fn serialize_seq(&self) -> SerializeSeq<&Self> {
            SerializeSeq::new(&self)
        }
    }
    impl<'a> KeyValues for Properties<'a> {
        fn serialize(&self, serializer: &mut dyn Serializer) {
            self.kvs.serialize(serializer);
            if let Some(parent) = self.parent {
                parent.serialize(serializer);
            }
        }
    }
    impl<'a> fmt::Debug for Properties<'a> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.debug_struct("Properties").finish()
        }
    }
    impl<'a> Default for Properties<'a> {
        fn default() -> Self {
            Properties::empty()
        }
    }
}
fn test_stuff() {
    {
        let properties = ::properties::Properties::empty();
        let value = &1;
        let adapter = ::properties::adapter::map::default(value);
        let kvs = ::properties::RawKeyValue("key", &adapter);
        let properties = ::properties::Properties::chained(&kvs, &properties);
    };
}
static mut LOGGER: &'static Log = &NopLogger;
static STATE: AtomicUsize = ATOMIC_USIZE_INIT;
const UNINITIALIZED: usize = 0;
const INITIALIZING: usize = 1;
const INITIALIZED: usize = 2;
static MAX_LOG_LEVEL_FILTER: AtomicUsize = ATOMIC_USIZE_INIT;
static LOG_LEVEL_NAMES: [&'static str; 6] = ["OFF", "ERROR", "WARN", "INFO", "DEBUG", "TRACE"];
static SET_LOGGER_ERROR: &'static str =
    "attempted to set a logger after the logging system was already initialized";
static LEVEL_PARSE_ERROR: &'static str =
    "attempted to convert a string that doesn\'t match an existing log level";
/// An enum representing the available verbosity levels of the logger.
///
/// Typical usage includes: checking if a certain `Level` is enabled with
/// [`log_enabled!`](macro.log_enabled.html), specifying the `Level` of
/// [`log!`](macro.log.html), and comparing a `Level` directly to a
/// [`LevelFilter`](enum.LevelFilter.html).
#[repr(usize)]
#[rustc_copy_clone_marker]
pub enum Level {
    /// The "error" level.
    ///
    /// Designates very serious errors.
    Error = 1,

    /// The "warn" level.
    ///
    /// Designates hazardous situations.
    Warn,

    /// The "info" level.
    ///
    /// Designates useful information.
    Info,

    /// The "debug" level.
    ///
    /// Designates lower priority information.
    Debug,

    /// The "trace" level.
    ///
    /// Designates very low priority, often extremely verbose, information.
    Trace,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::marker::Copy for Level {}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::cmp::Eq for Level {
    #[inline]
    #[doc(hidden)]
    fn assert_receiver_is_total_eq(&self) -> () {
        {}
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::fmt::Debug for Level {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match (&*self,) {
            (&Level::Error,) => {
                let mut debug_trait_builder = f.debug_tuple("Error");
                debug_trait_builder.finish()
            }
            (&Level::Warn,) => {
                let mut debug_trait_builder = f.debug_tuple("Warn");
                debug_trait_builder.finish()
            }
            (&Level::Info,) => {
                let mut debug_trait_builder = f.debug_tuple("Info");
                debug_trait_builder.finish()
            }
            (&Level::Debug,) => {
                let mut debug_trait_builder = f.debug_tuple("Debug");
                debug_trait_builder.finish()
            }
            (&Level::Trace,) => {
                let mut debug_trait_builder = f.debug_tuple("Trace");
                debug_trait_builder.finish()
            }
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::hash::Hash for Level {
    fn hash<__H: ::std::hash::Hasher>(&self, state: &mut __H) -> () {
        match (&*self,) {
            _ => ::std::hash::Hash::hash(
                &unsafe { ::std::intrinsics::discriminant_value(self) },
                state,
            ),
        }
    }
}
impl Clone for Level {
    #[inline]
    fn clone(&self) -> Level {
        *self
    }
}
impl PartialEq for Level {
    #[inline]
    fn eq(&self, other: &Level) -> bool {
        *self as usize == *other as usize
    }
}
impl PartialEq<LevelFilter> for Level {
    #[inline]
    fn eq(&self, other: &LevelFilter) -> bool {
        *self as usize == *other as usize
    }
}
impl PartialOrd for Level {
    #[inline]
    fn partial_cmp(&self, other: &Level) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialOrd<LevelFilter> for Level {
    #[inline]
    fn partial_cmp(&self, other: &LevelFilter) -> Option<cmp::Ordering> {
        Some((*self as usize).cmp(&(*other as usize)))
    }
}
impl Ord for Level {
    #[inline]
    fn cmp(&self, other: &Level) -> cmp::Ordering {
        (*self as usize).cmp(&(*other as usize))
    }
}
fn ok_or<T, E>(t: Option<T>, e: E) -> Result<T, E> {
    match t {
        Some(t) => Ok(t),
        None => Err(e),
    }
}
fn eq_ignore_ascii_case(a: &str, b: &str) -> bool {
    fn to_ascii_uppercase(c: u8) -> u8 {
        if c >= b'a' && c <= b'z' {
            c - b'a' + b'A'
        } else {
            c
        }
    }
    if a.len() == b.len() {
        a.bytes()
            .zip(b.bytes())
            .all(|(a, b)| to_ascii_uppercase(a) == to_ascii_uppercase(b))
    } else {
        false
    }
}
impl FromStr for Level {
    type Err = ParseLevelError;
    fn from_str(level: &str) -> Result<Level, Self::Err> {
        ok_or(
            LOG_LEVEL_NAMES
                .iter()
                .position(|&name| eq_ignore_ascii_case(name, level))
                .into_iter()
                .filter(|&idx| idx != 0)
                .map(|idx| Level::from_usize(idx).unwrap())
                .next(),
            ParseLevelError(()),
        )
    }
}
impl fmt::Display for Level {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.pad(LOG_LEVEL_NAMES[*self as usize])
    }
}
impl Level {
    fn from_usize(u: usize) -> Option<Level> {
        match u {
            1 => Some(Level::Error),
            2 => Some(Level::Warn),
            3 => Some(Level::Info),
            4 => Some(Level::Debug),
            5 => Some(Level::Trace),
            _ => None,
        }
    }
    /// Returns the most verbose logging level.
    #[inline]
    pub fn max() -> Level {
        Level::Trace
    }
    /// Converts the `Level` to the equivalent `LevelFilter`.
    #[inline]
    pub fn to_level_filter(&self) -> LevelFilter {
        LevelFilter::from_usize(*self as usize).unwrap()
    }
}
/// An enum representing the available verbosity level filters of the logger.
///
/// A `LevelFilter` may be compared directly to a [`Level`]. Use this type
/// to get and set the maximum log level with [`max_level()`] and [`set_max_level`].
///
/// [`Level`]: enum.Level.html
/// [`max_level()`]: fn.max_level.html
/// [`set_max_level`]: fn.set_max_level.html
#[repr(usize)]
#[rustc_copy_clone_marker]
pub enum LevelFilter {
    /// A level lower than all log levels.
    Off,

    /// Corresponds to the `Error` log level.
    Error,

    /// Corresponds to the `Warn` log level.
    Warn,

    /// Corresponds to the `Info` log level.
    Info,

    /// Corresponds to the `Debug` log level.
    Debug,

    /// Corresponds to the `Trace` log level.
    Trace,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::marker::Copy for LevelFilter {}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::cmp::Eq for LevelFilter {
    #[inline]
    #[doc(hidden)]
    fn assert_receiver_is_total_eq(&self) -> () {
        {}
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::fmt::Debug for LevelFilter {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match (&*self,) {
            (&LevelFilter::Off,) => {
                let mut debug_trait_builder = f.debug_tuple("Off");
                debug_trait_builder.finish()
            }
            (&LevelFilter::Error,) => {
                let mut debug_trait_builder = f.debug_tuple("Error");
                debug_trait_builder.finish()
            }
            (&LevelFilter::Warn,) => {
                let mut debug_trait_builder = f.debug_tuple("Warn");
                debug_trait_builder.finish()
            }
            (&LevelFilter::Info,) => {
                let mut debug_trait_builder = f.debug_tuple("Info");
                debug_trait_builder.finish()
            }
            (&LevelFilter::Debug,) => {
                let mut debug_trait_builder = f.debug_tuple("Debug");
                debug_trait_builder.finish()
            }
            (&LevelFilter::Trace,) => {
                let mut debug_trait_builder = f.debug_tuple("Trace");
                debug_trait_builder.finish()
            }
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::hash::Hash for LevelFilter {
    fn hash<__H: ::std::hash::Hasher>(&self, state: &mut __H) -> () {
        match (&*self,) {
            _ => ::std::hash::Hash::hash(
                &unsafe { ::std::intrinsics::discriminant_value(self) },
                state,
            ),
        }
    }
}
impl Clone for LevelFilter {
    #[inline]
    fn clone(&self) -> LevelFilter {
        *self
    }
}
impl PartialEq for LevelFilter {
    #[inline]
    fn eq(&self, other: &LevelFilter) -> bool {
        *self as usize == *other as usize
    }
}
impl PartialEq<Level> for LevelFilter {
    #[inline]
    fn eq(&self, other: &Level) -> bool {
        other.eq(self)
    }
}
impl PartialOrd for LevelFilter {
    #[inline]
    fn partial_cmp(&self, other: &LevelFilter) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialOrd<Level> for LevelFilter {
    #[inline]
    fn partial_cmp(&self, other: &Level) -> Option<cmp::Ordering> {
        other.partial_cmp(self).map(|x| x.reverse())
    }
}
impl Ord for LevelFilter {
    #[inline]
    fn cmp(&self, other: &LevelFilter) -> cmp::Ordering {
        (*self as usize).cmp(&(*other as usize))
    }
}
impl FromStr for LevelFilter {
    type Err = ParseLevelError;
    fn from_str(level: &str) -> Result<LevelFilter, Self::Err> {
        ok_or(
            LOG_LEVEL_NAMES
                .iter()
                .position(|&name| eq_ignore_ascii_case(name, level))
                .map(|p| LevelFilter::from_usize(p).unwrap()),
            ParseLevelError(()),
        )
    }
}
impl fmt::Display for LevelFilter {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_fmt(::std::fmt::Arguments::new_v1_formatted(
            &[""],
            &match (&LOG_LEVEL_NAMES[*self as usize],) {
                (arg0,) => [::std::fmt::ArgumentV1::new(arg0, ::std::fmt::Display::fmt)],
            },
            &[::std::fmt::rt::v1::Argument {
                position: ::std::fmt::rt::v1::Position::At(0usize),
                format: ::std::fmt::rt::v1::FormatSpec {
                    fill: ' ',
                    align: ::std::fmt::rt::v1::Alignment::Unknown,
                    flags: 0u32,
                    precision: ::std::fmt::rt::v1::Count::Implied,
                    width: ::std::fmt::rt::v1::Count::Implied,
                },
            }],
        ))
    }
}
impl LevelFilter {
    fn from_usize(u: usize) -> Option<LevelFilter> {
        match u {
            0 => Some(LevelFilter::Off),
            1 => Some(LevelFilter::Error),
            2 => Some(LevelFilter::Warn),
            3 => Some(LevelFilter::Info),
            4 => Some(LevelFilter::Debug),
            5 => Some(LevelFilter::Trace),
            _ => None,
        }
    }
    /// Returns the most verbose logging level filter.
    #[inline]
    pub fn max() -> LevelFilter {
        LevelFilter::Trace
    }
    /// Converts `self` to the equivalent `Level`.
    ///
    /// Returns `None` if `self` is `LevelFilter::Off`.
    #[inline]
    pub fn to_level(&self) -> Option<Level> {
        Level::from_usize(*self as usize)
    }
}
/// The "payload" of a log message.
///
/// # Use
///
/// `Record` structures are passed as parameters to the [`log`][method.log]
/// method of the [`Log`] trait. Logger implementors manipulate these
/// structures in order to display log messages. `Record`s are automatically
/// created by the [`log!`] macro and so are not seen by log users.
///
/// Note that the [`level()`] and [`target()`] accessors are equivalent to
/// `self.metadata().level()` and `self.metadata().target()` respectively.
/// These methods are provided as a convenience for users of this structure.
///
/// # Example
///
/// The following example shows a simple logger that displays the level,
/// module path, and message of any `Record` that is passed to it.
///
/// ```rust
/// # extern crate log;
/// struct SimpleLogger;
///
/// impl log::Log for SimpleLogger {
///    fn enabled(&self, metadata: &log::Metadata) -> bool {
///        true
///    }
///
///    fn log(&self, record: &log::Record) {
///        if !self.enabled(record.metadata()) {
///            return;
///        }
///
///        println!("{}:{} -- {}",
///                 record.level(),
///                 record.target(),
///                 record.args());
///    }
///    fn flush(&self) {}
/// }
/// ```
///
/// [method.log]: trait.Log.html#tymethod.log
/// [`Log`]: trait.Log.html
/// [`log!`]: macro.log.html
/// [`level()`]: struct.Record.html#method.level
/// [`target()`]: struct.Record.html#method.target
pub struct Record<'a> {
    header: Header<'a>,
    #[cfg(feature = "serde")]
    properties: properties::Properties<'a>,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<'a> ::std::clone::Clone for Record<'a> {
    #[inline]
    fn clone(&self) -> Record<'a> {
        match *self {
            Record {
                header: ref __self_0_0,
                properties: ref __self_0_1,
            } => Record {
                header: ::std::clone::Clone::clone(&(*__self_0_0)),
                properties: ::std::clone::Clone::clone(&(*__self_0_1)),
            },
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<'a> ::std::fmt::Debug for Record<'a> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            Record {
                header: ref __self_0_0,
                properties: ref __self_0_1,
            } => {
                let mut debug_trait_builder = f.debug_struct("Record");
                let _ = debug_trait_builder.field("header", &&(*__self_0_0));
                let _ = debug_trait_builder.field("properties", &&(*__self_0_1));
                debug_trait_builder.finish()
            }
        }
    }
}
struct Header<'a> {
    metadata: Metadata<'a>,
    args: fmt::Arguments<'a>,
    module_path: Option<&'a str>,
    file: Option<&'a str>,
    line: Option<u32>,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<'a> ::std::clone::Clone for Header<'a> {
    #[inline]
    fn clone(&self) -> Header<'a> {
        match *self {
            Header {
                metadata: ref __self_0_0,
                args: ref __self_0_1,
                module_path: ref __self_0_2,
                file: ref __self_0_3,
                line: ref __self_0_4,
            } => Header {
                metadata: ::std::clone::Clone::clone(&(*__self_0_0)),
                args: ::std::clone::Clone::clone(&(*__self_0_1)),
                module_path: ::std::clone::Clone::clone(&(*__self_0_2)),
                file: ::std::clone::Clone::clone(&(*__self_0_3)),
                line: ::std::clone::Clone::clone(&(*__self_0_4)),
            },
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<'a> ::std::fmt::Debug for Header<'a> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            Header {
                metadata: ref __self_0_0,
                args: ref __self_0_1,
                module_path: ref __self_0_2,
                file: ref __self_0_3,
                line: ref __self_0_4,
            } => {
                let mut debug_trait_builder = f.debug_struct("Header");
                let _ = debug_trait_builder.field("metadata", &&(*__self_0_0));
                let _ = debug_trait_builder.field("args", &&(*__self_0_1));
                let _ = debug_trait_builder.field("module_path", &&(*__self_0_2));
                let _ = debug_trait_builder.field("file", &&(*__self_0_3));
                let _ = debug_trait_builder.field("line", &&(*__self_0_4));
                debug_trait_builder.finish()
            }
        }
    }
}
impl<'a> Record<'a> {
    /// Returns a new builder.
    #[inline]
    pub fn builder() -> RecordBuilder<'a> {
        RecordBuilder::new()
    }
    /// The message body.
    #[inline]
    pub fn args(&self) -> &fmt::Arguments<'a> {
        &self.header.args
    }
    /// Metadata about the log directive.
    #[inline]
    pub fn metadata(&self) -> &Metadata<'a> {
        &self.header.metadata
    }
    /// The verbosity level of the message.
    #[inline]
    pub fn level(&self) -> Level {
        self.header.metadata.level()
    }
    /// The name of the target of the directive.
    #[inline]
    pub fn target(&self) -> &'a str {
        self.header.metadata.target()
    }
    /// The module path of the message.
    #[inline]
    pub fn module_path(&self) -> Option<&'a str> {
        self.header.module_path
    }
    /// The source file containing the message.
    #[inline]
    pub fn file(&self) -> Option<&'a str> {
        self.header.file
    }
    /// The line containing the message.
    #[inline]
    pub fn line(&self) -> Option<u32> {
        self.header.line
    }
    /// Get a new borrowed record with the additional properties.
    #[inline]
    #[cfg(feature = "serde")]
    pub fn push<'b>(&'b self, properties: &'b dyn properties::KeyValues) -> Record<'b> {
        Record {
            header: self.header.clone(),
            properties: properties::Properties::chained(properties, &self.properties),
        }
    }
    /// The properties attached to this record.
    ///
    /// Properties aren't guaranteed to be unique (the same key may be repeated with different values).
    #[inline]
    #[cfg(feature = "serde")]
    pub fn properties(&self) -> Option<&properties::Properties> {
        Some(&self.properties)
    }
}
/// Builder for [`Record`](struct.Record.html).
///
/// Typically should only be used by log library creators or for testing and "shim loggers".
/// The `RecordBuilder` can set the different parameters of `Record` object, and returns
/// the created object when `build` is called.
///
/// # Examples
///
///
/// ```rust
/// use log::{Level, Record};
///
/// let record = Record::builder()
///                 .args(format_args!("Error!"))
///                 .level(Level::Error)
///                 .target("myApp")
///                 .file(Some("server.rs"))
///                 .line(Some(144))
///                 .module_path(Some("server"))
///                 .build();
/// ```
///
/// Alternatively, use [`MetadataBuilder`](struct.MetadataBuilder.html):
///
/// ```rust
/// use log::{Record, Level, MetadataBuilder};
///
/// let error_metadata = MetadataBuilder::new()
///                         .target("myApp")
///                         .level(Level::Error)
///                         .build();
///
/// let record = Record::builder()
///                 .metadata(error_metadata)
///                 .args(format_args!("Error!"))
///                 .line(Some(433))
///                 .file(Some("app.rs"))
///                 .module_path(Some("server"))
///                 .build();
/// ```
pub struct RecordBuilder<'a> {
    record: Record<'a>,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<'a> ::std::fmt::Debug for RecordBuilder<'a> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            RecordBuilder {
                record: ref __self_0_0,
            } => {
                let mut debug_trait_builder = f.debug_struct("RecordBuilder");
                let _ = debug_trait_builder.field("record", &&(*__self_0_0));
                debug_trait_builder.finish()
            }
        }
    }
}
impl<'a> RecordBuilder<'a> {
    /// Construct new `RecordBuilder`.
    ///
    /// The default options are:
    ///
    /// - `args`: [`format_args!("")`]
    /// - `metadata`: [`Metadata::builder().build()`]
    /// - `module_path`: `None`
    /// - `file`: `None`
    /// - `line`: `None`
    ///
    /// [`format_args!("")`]: https://doc.rust-lang.org/std/macro.format_args.html
    /// [`Metadata::builder().build()`]: struct.MetadataBuilder.html#method.build
    #[inline]
    pub fn new() -> RecordBuilder<'a> {
        RecordBuilder {
            record: Record {
                header: Header {
                    args: ::std::fmt::Arguments::new_v1(
                        &[],
                        &match () {
                            () => [],
                        },
                    ),
                    metadata: Metadata::builder().build(),
                    module_path: None,
                    file: None,
                    line: None,
                },
                properties: Default::default(),
            },
        }
    }
    /// Set [`args`](struct.Record.html#method.args).
    #[inline]
    pub fn args(&mut self, args: fmt::Arguments<'a>) -> &mut RecordBuilder<'a> {
        self.record.header.args = args;
        self
    }
    /// Set [`metadata`](struct.Record.html#method.metadata). Construct a `Metadata` object with [`MetadataBuilder`](struct.MetadataBuilder.html).
    #[inline]
    pub fn metadata(&mut self, metadata: Metadata<'a>) -> &mut RecordBuilder<'a> {
        self.record.header.metadata = metadata;
        self
    }
    /// Set [`Metadata::level`](struct.Metadata.html#method.level).
    #[inline]
    pub fn level(&mut self, level: Level) -> &mut RecordBuilder<'a> {
        self.record.header.metadata.level = level;
        self
    }
    /// Set [`Metadata::target`](struct.Metadata.html#method.target)
    #[inline]
    pub fn target(&mut self, target: &'a str) -> &mut RecordBuilder<'a> {
        self.record.header.metadata.target = target;
        self
    }
    /// Set [`module_path`](struct.Record.html#method.module_path)
    #[inline]
    pub fn module_path(&mut self, path: Option<&'a str>) -> &mut RecordBuilder<'a> {
        self.record.header.module_path = path;
        self
    }
    /// Set [`file`](struct.Record.html#method.file)
    #[inline]
    pub fn file(&mut self, file: Option<&'a str>) -> &mut RecordBuilder<'a> {
        self.record.header.file = file;
        self
    }
    /// Set [`line`](struct.Record.html#method.line)
    #[inline]
    pub fn line(&mut self, line: Option<u32>) -> &mut RecordBuilder<'a> {
        self.record.header.line = line;
        self
    }
    /// Set properties
    #[inline]
    #[cfg(feature = "serde")]
    pub fn properties(
        &mut self,
        properties: &'a dyn properties::KeyValues,
    ) -> &mut RecordBuilder<'a> {
        self.record.properties = properties::Properties::root(properties);
        self
    }
    /// Invoke the builder and return a `Record`
    #[inline]
    pub fn build(&self) -> Record<'a> {
        self.record.clone()
    }
}
/// Metadata about a log message.
///
/// # Use
///
/// `Metadata` structs are created when users of the library use
/// logging macros.
///
/// They are consumed by implementations of the `Log` trait in the
/// `enabled` method.
///
/// `Record`s use `Metadata` to determine the log message's severity
/// and target.
///
/// Users should use the `log_enabled!` macro in their code to avoid
/// constructing expensive log messages.
///
/// # Examples
///
/// ```rust
/// # #[macro_use]
/// # extern crate log;
/// #
/// use log::{Record, Level, Metadata};
///
/// struct MyLogger;
///
/// impl log::Log for MyLogger {
///     fn enabled(&self, metadata: &Metadata) -> bool {
///         metadata.level() <= Level::Info
///     }
///
///     fn log(&self, record: &Record) {
///         if self.enabled(record.metadata()) {
///             println!("{} - {}", record.level(), record.args());
///         }
///     }
///     fn flush(&self) {}
/// }
///
/// # fn main(){}
/// ```
#[structural_match]
pub struct Metadata<'a> {
    level: Level,
    target: &'a str,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<'a> ::std::clone::Clone for Metadata<'a> {
    #[inline]
    fn clone(&self) -> Metadata<'a> {
        match *self {
            Metadata {
                level: ref __self_0_0,
                target: ref __self_0_1,
            } => Metadata {
                level: ::std::clone::Clone::clone(&(*__self_0_0)),
                target: ::std::clone::Clone::clone(&(*__self_0_1)),
            },
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<'a> ::std::cmp::Eq for Metadata<'a> {
    #[inline]
    #[doc(hidden)]
    fn assert_receiver_is_total_eq(&self) -> () {
        {
            let _: ::std::cmp::AssertParamIsEq<Level>;
            let _: ::std::cmp::AssertParamIsEq<&'a str>;
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<'a> ::std::cmp::PartialEq for Metadata<'a> {
    #[inline]
    fn eq(&self, other: &Metadata<'a>) -> bool {
        match *other {
            Metadata {
                level: ref __self_1_0,
                target: ref __self_1_1,
            } => match *self {
                Metadata {
                    level: ref __self_0_0,
                    target: ref __self_0_1,
                } => (*__self_0_0) == (*__self_1_0) && (*__self_0_1) == (*__self_1_1),
            },
        }
    }
    #[inline]
    fn ne(&self, other: &Metadata<'a>) -> bool {
        match *other {
            Metadata {
                level: ref __self_1_0,
                target: ref __self_1_1,
            } => match *self {
                Metadata {
                    level: ref __self_0_0,
                    target: ref __self_0_1,
                } => (*__self_0_0) != (*__self_1_0) || (*__self_0_1) != (*__self_1_1),
            },
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<'a> ::std::cmp::Ord for Metadata<'a> {
    #[inline]
    fn cmp(&self, other: &Metadata<'a>) -> ::std::cmp::Ordering {
        match *other {
            Metadata {
                level: ref __self_1_0,
                target: ref __self_1_1,
            } => match *self {
                Metadata {
                    level: ref __self_0_0,
                    target: ref __self_0_1,
                } => match ::std::cmp::Ord::cmp(&(*__self_0_0), &(*__self_1_0)) {
                    ::std::cmp::Ordering::Equal => {
                        match ::std::cmp::Ord::cmp(&(*__self_0_1), &(*__self_1_1)) {
                            ::std::cmp::Ordering::Equal => ::std::cmp::Ordering::Equal,
                            cmp => cmp,
                        }
                    }
                    cmp => cmp,
                },
            },
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<'a> ::std::cmp::PartialOrd for Metadata<'a> {
    #[inline]
    fn partial_cmp(&self, other: &Metadata<'a>) -> ::std::option::Option<::std::cmp::Ordering> {
        match *other {
            Metadata {
                level: ref __self_1_0,
                target: ref __self_1_1,
            } => match *self {
                Metadata {
                    level: ref __self_0_0,
                    target: ref __self_0_1,
                } => match ::std::cmp::PartialOrd::partial_cmp(&(*__self_0_0), &(*__self_1_0)) {
                    ::std::option::Option::Some(::std::cmp::Ordering::Equal) => {
                        match ::std::cmp::PartialOrd::partial_cmp(&(*__self_0_1), &(*__self_1_1)) {
                            ::std::option::Option::Some(::std::cmp::Ordering::Equal) => {
                                ::std::option::Option::Some(::std::cmp::Ordering::Equal)
                            }
                            cmp => cmp,
                        }
                    }
                    cmp => cmp,
                },
            },
        }
    }
    #[inline]
    fn lt(&self, other: &Metadata<'a>) -> bool {
        match *other {
            Metadata {
                level: ref __self_1_0,
                target: ref __self_1_1,
            } => match *self {
                Metadata {
                    level: ref __self_0_0,
                    target: ref __self_0_1,
                } => {
                    ::std::cmp::Ordering::then_with(
                        ::std::option::Option::unwrap_or(
                            ::std::cmp::PartialOrd::partial_cmp(&(*__self_0_0), &(*__self_1_0)),
                            ::std::cmp::Ordering::Equal,
                        ),
                        || {
                            ::std::option::Option::unwrap_or(
                                ::std::cmp::PartialOrd::partial_cmp(&(*__self_0_1), &(*__self_1_1)),
                                ::std::cmp::Ordering::Greater,
                            )
                        },
                    ) == ::std::cmp::Ordering::Less
                }
            },
        }
    }
    #[inline]
    fn le(&self, other: &Metadata<'a>) -> bool {
        match *other {
            Metadata {
                level: ref __self_1_0,
                target: ref __self_1_1,
            } => match *self {
                Metadata {
                    level: ref __self_0_0,
                    target: ref __self_0_1,
                } => {
                    ::std::cmp::Ordering::then_with(
                        ::std::option::Option::unwrap_or(
                            ::std::cmp::PartialOrd::partial_cmp(&(*__self_0_0), &(*__self_1_0)),
                            ::std::cmp::Ordering::Equal,
                        ),
                        || {
                            ::std::option::Option::unwrap_or(
                                ::std::cmp::PartialOrd::partial_cmp(&(*__self_0_1), &(*__self_1_1)),
                                ::std::cmp::Ordering::Greater,
                            )
                        },
                    ) != ::std::cmp::Ordering::Greater
                }
            },
        }
    }
    #[inline]
    fn gt(&self, other: &Metadata<'a>) -> bool {
        match *other {
            Metadata {
                level: ref __self_1_0,
                target: ref __self_1_1,
            } => match *self {
                Metadata {
                    level: ref __self_0_0,
                    target: ref __self_0_1,
                } => {
                    ::std::cmp::Ordering::then_with(
                        ::std::option::Option::unwrap_or(
                            ::std::cmp::PartialOrd::partial_cmp(&(*__self_0_0), &(*__self_1_0)),
                            ::std::cmp::Ordering::Equal,
                        ),
                        || {
                            ::std::option::Option::unwrap_or(
                                ::std::cmp::PartialOrd::partial_cmp(&(*__self_0_1), &(*__self_1_1)),
                                ::std::cmp::Ordering::Less,
                            )
                        },
                    ) == ::std::cmp::Ordering::Greater
                }
            },
        }
    }
    #[inline]
    fn ge(&self, other: &Metadata<'a>) -> bool {
        match *other {
            Metadata {
                level: ref __self_1_0,
                target: ref __self_1_1,
            } => match *self {
                Metadata {
                    level: ref __self_0_0,
                    target: ref __self_0_1,
                } => {
                    ::std::cmp::Ordering::then_with(
                        ::std::option::Option::unwrap_or(
                            ::std::cmp::PartialOrd::partial_cmp(&(*__self_0_0), &(*__self_1_0)),
                            ::std::cmp::Ordering::Equal,
                        ),
                        || {
                            ::std::option::Option::unwrap_or(
                                ::std::cmp::PartialOrd::partial_cmp(&(*__self_0_1), &(*__self_1_1)),
                                ::std::cmp::Ordering::Less,
                            )
                        },
                    ) != ::std::cmp::Ordering::Less
                }
            },
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<'a> ::std::hash::Hash for Metadata<'a> {
    fn hash<__H: ::std::hash::Hasher>(&self, state: &mut __H) -> () {
        match *self {
            Metadata {
                level: ref __self_0_0,
                target: ref __self_0_1,
            } => {
                ::std::hash::Hash::hash(&(*__self_0_0), state);
                ::std::hash::Hash::hash(&(*__self_0_1), state)
            }
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<'a> ::std::fmt::Debug for Metadata<'a> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            Metadata {
                level: ref __self_0_0,
                target: ref __self_0_1,
            } => {
                let mut debug_trait_builder = f.debug_struct("Metadata");
                let _ = debug_trait_builder.field("level", &&(*__self_0_0));
                let _ = debug_trait_builder.field("target", &&(*__self_0_1));
                debug_trait_builder.finish()
            }
        }
    }
}
impl<'a> Metadata<'a> {
    /// Returns a new builder.
    #[inline]
    pub fn builder() -> MetadataBuilder<'a> {
        MetadataBuilder::new()
    }
    /// The verbosity level of the message.
    #[inline]
    pub fn level(&self) -> Level {
        self.level
    }
    /// The name of the target of the directive.
    #[inline]
    pub fn target(&self) -> &'a str {
        self.target
    }
}
/// Builder for [`Metadata`](struct.Metadata.html).
///
/// Typically should only be used by log library creators or for testing and "shim loggers".
/// The `MetadataBuilder` can set the different parameters of a `Metadata` object, and returns
/// the created object when `build` is called.
///
/// # Example
///
/// ```rust
/// let target = "myApp";
/// use log::{Level, MetadataBuilder};
/// let metadata = MetadataBuilder::new()
///                     .level(Level::Debug)
///                     .target(target)
///                     .build();
/// ```
#[structural_match]
pub struct MetadataBuilder<'a> {
    metadata: Metadata<'a>,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<'a> ::std::cmp::Eq for MetadataBuilder<'a> {
    #[inline]
    #[doc(hidden)]
    fn assert_receiver_is_total_eq(&self) -> () {
        {
            let _: ::std::cmp::AssertParamIsEq<Metadata<'a>>;
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<'a> ::std::cmp::PartialEq for MetadataBuilder<'a> {
    #[inline]
    fn eq(&self, other: &MetadataBuilder<'a>) -> bool {
        match *other {
            MetadataBuilder {
                metadata: ref __self_1_0,
            } => match *self {
                MetadataBuilder {
                    metadata: ref __self_0_0,
                } => (*__self_0_0) == (*__self_1_0),
            },
        }
    }
    #[inline]
    fn ne(&self, other: &MetadataBuilder<'a>) -> bool {
        match *other {
            MetadataBuilder {
                metadata: ref __self_1_0,
            } => match *self {
                MetadataBuilder {
                    metadata: ref __self_0_0,
                } => (*__self_0_0) != (*__self_1_0),
            },
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<'a> ::std::cmp::Ord for MetadataBuilder<'a> {
    #[inline]
    fn cmp(&self, other: &MetadataBuilder<'a>) -> ::std::cmp::Ordering {
        match *other {
            MetadataBuilder {
                metadata: ref __self_1_0,
            } => match *self {
                MetadataBuilder {
                    metadata: ref __self_0_0,
                } => match ::std::cmp::Ord::cmp(&(*__self_0_0), &(*__self_1_0)) {
                    ::std::cmp::Ordering::Equal => ::std::cmp::Ordering::Equal,
                    cmp => cmp,
                },
            },
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<'a> ::std::cmp::PartialOrd for MetadataBuilder<'a> {
    #[inline]
    fn partial_cmp(
        &self,
        other: &MetadataBuilder<'a>,
    ) -> ::std::option::Option<::std::cmp::Ordering> {
        match *other {
            MetadataBuilder {
                metadata: ref __self_1_0,
            } => match *self {
                MetadataBuilder {
                    metadata: ref __self_0_0,
                } => match ::std::cmp::PartialOrd::partial_cmp(&(*__self_0_0), &(*__self_1_0)) {
                    ::std::option::Option::Some(::std::cmp::Ordering::Equal) => {
                        ::std::option::Option::Some(::std::cmp::Ordering::Equal)
                    }
                    cmp => cmp,
                },
            },
        }
    }
    #[inline]
    fn lt(&self, other: &MetadataBuilder<'a>) -> bool {
        match *other {
            MetadataBuilder {
                metadata: ref __self_1_0,
            } => match *self {
                MetadataBuilder {
                    metadata: ref __self_0_0,
                } => {
                    ::std::option::Option::unwrap_or(
                        ::std::cmp::PartialOrd::partial_cmp(&(*__self_0_0), &(*__self_1_0)),
                        ::std::cmp::Ordering::Greater,
                    ) == ::std::cmp::Ordering::Less
                }
            },
        }
    }
    #[inline]
    fn le(&self, other: &MetadataBuilder<'a>) -> bool {
        match *other {
            MetadataBuilder {
                metadata: ref __self_1_0,
            } => match *self {
                MetadataBuilder {
                    metadata: ref __self_0_0,
                } => {
                    ::std::option::Option::unwrap_or(
                        ::std::cmp::PartialOrd::partial_cmp(&(*__self_0_0), &(*__self_1_0)),
                        ::std::cmp::Ordering::Greater,
                    ) != ::std::cmp::Ordering::Greater
                }
            },
        }
    }
    #[inline]
    fn gt(&self, other: &MetadataBuilder<'a>) -> bool {
        match *other {
            MetadataBuilder {
                metadata: ref __self_1_0,
            } => match *self {
                MetadataBuilder {
                    metadata: ref __self_0_0,
                } => {
                    ::std::option::Option::unwrap_or(
                        ::std::cmp::PartialOrd::partial_cmp(&(*__self_0_0), &(*__self_1_0)),
                        ::std::cmp::Ordering::Less,
                    ) == ::std::cmp::Ordering::Greater
                }
            },
        }
    }
    #[inline]
    fn ge(&self, other: &MetadataBuilder<'a>) -> bool {
        match *other {
            MetadataBuilder {
                metadata: ref __self_1_0,
            } => match *self {
                MetadataBuilder {
                    metadata: ref __self_0_0,
                } => {
                    ::std::option::Option::unwrap_or(
                        ::std::cmp::PartialOrd::partial_cmp(&(*__self_0_0), &(*__self_1_0)),
                        ::std::cmp::Ordering::Less,
                    ) != ::std::cmp::Ordering::Less
                }
            },
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<'a> ::std::hash::Hash for MetadataBuilder<'a> {
    fn hash<__H: ::std::hash::Hasher>(&self, state: &mut __H) -> () {
        match *self {
            MetadataBuilder {
                metadata: ref __self_0_0,
            } => ::std::hash::Hash::hash(&(*__self_0_0), state),
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl<'a> ::std::fmt::Debug for MetadataBuilder<'a> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            MetadataBuilder {
                metadata: ref __self_0_0,
            } => {
                let mut debug_trait_builder = f.debug_struct("MetadataBuilder");
                let _ = debug_trait_builder.field("metadata", &&(*__self_0_0));
                debug_trait_builder.finish()
            }
        }
    }
}
impl<'a> MetadataBuilder<'a> {
    /// Construct a new `MetadataBuilder`.
    ///
    /// The default options are:
    ///
    /// - `level`: `Level::Info`
    /// - `target`: `""`
    #[inline]
    pub fn new() -> MetadataBuilder<'a> {
        MetadataBuilder {
            metadata: Metadata {
                level: Level::Info,
                target: "",
            },
        }
    }
    /// Setter for [`level`](struct.Metadata.html#method.level).
    #[inline]
    pub fn level(&mut self, arg: Level) -> &mut MetadataBuilder<'a> {
        self.metadata.level = arg;
        self
    }
    /// Setter for [`target`](struct.Metadata.html#method.target).
    #[inline]
    pub fn target(&mut self, target: &'a str) -> &mut MetadataBuilder<'a> {
        self.metadata.target = target;
        self
    }
    /// Returns a `Metadata` object.
    #[inline]
    pub fn build(&self) -> Metadata<'a> {
        self.metadata.clone()
    }
}
/// A trait encapsulating the operations required of a logger.
pub trait Log: Sync + Send {
    /// Determines if a log message with the specified metadata would be
    /// logged.
    ///
    /// This is used by the `log_enabled!` macro to allow callers to avoid
    /// expensive computation of log message arguments if the message would be
    /// discarded anyway.
    fn enabled(&self, metadata: &Metadata) -> bool;
    /// Logs the `Record`.
    ///
    /// Note that `enabled` is *not* necessarily called before this method.
    /// Implementations of `log` should perform all necessary filtering
    /// internally.
    fn log(&self, record: &Record);
    /// Flushes any buffered records.
    fn flush(&self);
}
struct NopLogger;
impl Log for NopLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        false
    }
    fn log(&self, _: &Record) {}
    fn flush(&self) {}
}
/// Sets the global maximum log level.
///
/// Generally, this should only be called by the active logging implementation.
#[inline]
pub fn set_max_level(level: LevelFilter) {
    MAX_LOG_LEVEL_FILTER.store(level as usize, Ordering::SeqCst)
}
/// Returns the current maximum log level.
///
/// The [`log!`], [`error!`], [`warn!`], [`info!`], [`debug!`], and [`trace!`] macros check
/// this value and discard any message logged at a higher level. The maximum
/// log level is set by the [`set_max_level`] function.
///
/// [`log!`]: macro.log.html
/// [`error!`]: macro.error.html
/// [`warn!`]: macro.warn.html
/// [`info!`]: macro.info.html
/// [`debug!`]: macro.debug.html
/// [`trace!`]: macro.trace.html
/// [`set_max_level`]: fn.set_max_level.html
#[inline(always)]
pub fn max_level() -> LevelFilter {
    unsafe { mem::transmute(MAX_LOG_LEVEL_FILTER.load(Ordering::Relaxed)) }
}
/// Sets the global logger to a `Box<Log>`.
///
/// This is a simple convenience wrapper over `set_logger`, which takes a
/// `Box<Log>` rather than a `&'static Log`. See the documentation for
/// [`set_logger`] for more details.
///
/// Requires the `std` feature.
///
/// # Errors
///
/// An error is returned if a logger has already been set.
///
/// [`set_logger`]: fn.set_logger.html
#[cfg(feature = "std")]
pub fn set_boxed_logger(logger: Box<Log>) -> Result<(), SetLoggerError> {
    set_logger_inner(|| unsafe { &*Box::into_raw(logger) })
}
/// Sets the global logger to a `&'static Log`.
///
/// This function may only be called once in the lifetime of a program. Any log
/// events that occur before the call to `set_logger` completes will be ignored.
///
/// This function does not typically need to be called manually. Logger
/// implementations should provide an initialization method that installs the
/// logger internally.
///
/// # Errors
///
/// An error is returned if a logger has already been set.
///
/// # Examples
///
/// ```rust
/// # #[macro_use]
/// # extern crate log;
/// #
/// use log::{Record, Level, Metadata, LevelFilter};
///
/// static MY_LOGGER: MyLogger = MyLogger;
///
/// struct MyLogger;
///
/// impl log::Log for MyLogger {
///     fn enabled(&self, metadata: &Metadata) -> bool {
///         metadata.level() <= Level::Info
///     }
///
///     fn log(&self, record: &Record) {
///         if self.enabled(record.metadata()) {
///             println!("{} - {}", record.level(), record.args());
///         }
///     }
///     fn flush(&self) {}
/// }
///
/// # fn main(){
/// log::set_logger(&MY_LOGGER).unwrap();
/// log::set_max_level(LevelFilter::Info);
///
/// info!("hello log");
/// warn!("warning");
/// error!("oops");
/// # }
/// ```
pub fn set_logger(logger: &'static Log) -> Result<(), SetLoggerError> {
    set_logger_inner(|| logger)
}
fn set_logger_inner<F>(make_logger: F) -> Result<(), SetLoggerError>
where
    F: FnOnce() -> &'static Log,
{
    unsafe {
        if STATE.compare_and_swap(UNINITIALIZED, INITIALIZING, Ordering::SeqCst) != UNINITIALIZED {
            return Err(SetLoggerError(()));
        }
        LOGGER = make_logger();
        STATE.store(INITIALIZED, Ordering::SeqCst);
        Ok(())
    }
}
/// The type returned by [`set_logger`] if [`set_logger`] has already been called.
///
/// [`set_logger`]: fn.set_logger.html
#[allow(missing_copy_implementations)]
pub struct SetLoggerError(());
#[automatically_derived]
#[allow(unused_qualifications)]
#[allow(missing_copy_implementations)]
impl ::std::fmt::Debug for SetLoggerError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            SetLoggerError(ref __self_0_0) => {
                let mut debug_trait_builder = f.debug_tuple("SetLoggerError");
                let _ = debug_trait_builder.field(&&(*__self_0_0));
                debug_trait_builder.finish()
            }
        }
    }
}
impl fmt::Display for SetLoggerError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(SET_LOGGER_ERROR)
    }
}
#[cfg(feature = "std")]
impl error::Error for SetLoggerError {
    fn description(&self) -> &str {
        SET_LOGGER_ERROR
    }
}
/// The type returned by [`from_str`] when the string doesn't match any of the log levels.
///
/// [`from_str`]: https://doc.rust-lang.org/std/str/trait.FromStr.html#tymethod.from_str
#[allow(missing_copy_implementations)]
pub struct ParseLevelError(());
#[automatically_derived]
#[allow(unused_qualifications)]
#[allow(missing_copy_implementations)]
impl ::std::fmt::Debug for ParseLevelError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            ParseLevelError(ref __self_0_0) => {
                let mut debug_trait_builder = f.debug_tuple("ParseLevelError");
                let _ = debug_trait_builder.field(&&(*__self_0_0));
                debug_trait_builder.finish()
            }
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
#[allow(missing_copy_implementations)]
impl ::std::cmp::PartialEq for ParseLevelError {
    #[inline]
    fn eq(&self, other: &ParseLevelError) -> bool {
        match *other {
            ParseLevelError(ref __self_1_0) => match *self {
                ParseLevelError(ref __self_0_0) => (*__self_0_0) == (*__self_1_0),
            },
        }
    }
    #[inline]
    fn ne(&self, other: &ParseLevelError) -> bool {
        match *other {
            ParseLevelError(ref __self_1_0) => match *self {
                ParseLevelError(ref __self_0_0) => (*__self_0_0) != (*__self_1_0),
            },
        }
    }
}
impl fmt::Display for ParseLevelError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(LEVEL_PARSE_ERROR)
    }
}
#[cfg(feature = "std")]
impl error::Error for ParseLevelError {
    fn description(&self) -> &str {
        LEVEL_PARSE_ERROR
    }
}
/// Returns a reference to the logger.
///
/// If a logger has not been set, a no-op implementation is returned.
pub fn logger() -> &'static Log {
    unsafe {
        if STATE.load(Ordering::SeqCst) != INITIALIZED {
            static NOP: NopLogger = NopLogger;
            &NOP
        } else {
            LOGGER
        }
    }
}
/// The statically resolved maximum log level.
///
/// See the crate level documentation for information on how to configure this.
///
/// This value is checked by the log macros, but not by the `Log`ger returned by
/// the [`logger`] function. Code that manually calls functions on that value
/// should compare the level against this value.
///
/// [`logger`]: fn.logger.html
pub const STATIC_MAX_LEVEL: LevelFilter = MAX_LEVEL_INNER;
#[cfg(
    all(
        not(
            any(
                all(not(debug_assertions), feature = "release_max_level_off"),
                all(not(debug_assertions), feature = "release_max_level_error"),
                all(not(debug_assertions), feature = "release_max_level_warn"),
                all(not(debug_assertions), feature = "release_max_level_info"),
                all(not(debug_assertions), feature = "release_max_level_debug"),
                all(not(debug_assertions), feature = "release_max_level_trace"),
                feature = "max_level_off",
                feature = "max_level_error",
                feature = "max_level_warn",
                feature = "max_level_info",
                feature = "max_level_debug"
            )
        )
    )
)]
const MAX_LEVEL_INNER: LevelFilter = LevelFilter::Trace;
