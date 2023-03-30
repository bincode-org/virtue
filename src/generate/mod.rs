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

mod gen_struct;
mod generate_fn;
mod generate_mod;
mod generator;
mod r#impl;
mod impl_for;
mod stream_builder;

use crate::{
    parse::{GenericConstraints, Generics},
    prelude::Ident,
};

pub use self::gen_struct::GenStruct;
pub use self::generate_fn::{FnBuilder, FnSelfArg};
pub use self::generate_mod::GenerateMod;
pub use self::generator::Generator;
pub use self::impl_for::{GenConst, ImplFor};
pub use self::r#impl::Impl;
pub use self::stream_builder::{PushParseError, StreamBuilder};

/// Helper trait to make it possible to nest several builders. Internal use only.
#[allow(missing_docs)]
pub trait Parent {
    fn append(&mut self, builder: StreamBuilder);
    fn name(&self) -> &Ident;
    fn generics(&self) -> Option<&Generics>;
    fn generic_constraints(&self) -> Option<&GenericConstraints>;
}
