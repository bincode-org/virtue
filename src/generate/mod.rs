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

mod gen_enum;
mod gen_struct;
mod generate_item;
mod generate_mod;
mod generator;
mod r#impl;
mod impl_for;
mod stream_builder;

use crate::{
    parse::{GenericConstraints, Generics},
    prelude::Ident,
};
use std::fmt;

pub use self::gen_enum::GenEnum;
pub use self::gen_struct::GenStruct;
pub use self::generate_item::{FnBuilder, FnSelfArg, GenConst};
pub use self::generate_mod::GenerateMod;
pub use self::generator::Generator;
pub use self::impl_for::ImplFor;
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

/// Helper enum to differentiate between a [`Ident`] or a [`String`].
#[allow(missing_docs)]
pub enum StringOrIdent {
    String(String),
    // Note that when this is a `string` this could be much more than a single ident.
    // Therefor you should never use [`StreamBuilder`]`.ident_str(StringOrIdent.to_string())`, but instead use `.push_parsed(StringOrIdent.to_string())?`.
    Ident(Ident),
}

impl fmt::Display for StringOrIdent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String(s) => s.fmt(f),
            Self::Ident(i) => i.fmt(f),
        }
    }
}

impl From<String> for StringOrIdent {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}
impl From<Ident> for StringOrIdent {
    fn from(i: Ident) -> Self {
        Self::Ident(i)
    }
}
impl<'a> From<&'a str> for StringOrIdent {
    fn from(s: &'a str) -> Self {
        Self::String(s.to_owned())
    }
}
