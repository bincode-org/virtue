//! Code to help generate functions.
//!
//! The structure is:
//!
//! - [`Generator`]
//!   - `.impl_for()`: [`ImplFor`]
//!     - `.generate_fn()`: [`FnBuilder`]
//!       - `.body(|builder| { .. })`: [`StreamBuilder`]
//!
//! Afterwards, [`Generator::finish()`] **must** be called to take out the [`TokenStream`] produced.
//!
//! [`Generator::finish()`]: struct.Generator.html#method.finish
//! [`TokenStream`]: ../prelude/struct.TokenStream.html

mod generate_fn;
mod generator;
mod impl_for;
mod stream_builder;

pub use self::generate_fn::{FnBuilder, FnSelfArg};
pub use self::generator::Generator;
pub use self::impl_for::ImplFor;
pub use self::stream_builder::{PushParseError, StreamBuilder};
