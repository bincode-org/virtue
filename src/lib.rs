#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

mod error;

pub mod generate;
pub mod parse;
pub mod utils;

/// Result alias for virtue's errors
pub type Result<T = ()> = std::result::Result<T, Error>;

pub use self::error::Error;

/// Useful includes
pub mod prelude {
    pub use crate::generate::{FnSelfArg, Generator, StreamBuilder};
    pub use crate::parse::{
        AttributeAccess, Body, EnumVariant, Fields, FromAttribute, Parse, UnnamedField,
    };
    pub use crate::{Error, Result};

    #[cfg(test)]
    pub use proc_macro2::*;

    #[cfg(not(test))]
    extern crate proc_macro;
    #[cfg(not(test))]
    pub use proc_macro::*;
}

#[cfg(test)]
pub(crate) fn token_stream(
    s: &str,
) -> std::iter::Peekable<impl Iterator<Item = proc_macro2::TokenTree>> {
    use std::str::FromStr;

    let stream = proc_macro2::TokenStream::from_str(s)
        .unwrap_or_else(|e| panic!("Could not parse code: {:?}\n{:?}", s, e));
    stream.into_iter().peekable()
}
