//! # Virtue, a sinless derive macro helper
//!
//! ## Goals
//!
//! - Zero dependencies, so fast compile times
//! - No other dependencies needed
//! - Declarative code generation
//! - As much typesystem checking as possible
//! - Build for modern rust: 1.57 and up
//! - Build with popular crates in mind:
//!   - [bincode](https://docs.rs/bincode)
//! - Will always respect semver. Minor releases will never have:
//!   - Breaking API changes
//!   - MSRV changes
//!
//! ## Example
//!
//! ```ignore
//! use virtue::prelude::*;
//!
//! #[proc_macro_derive(YourDerive, attributes(some, attributes, go, here))]
//! pub fn derive_your_derive(input: TokenStream) -> TokenStream {
//!     derive_your_derive_inner(input)
//!         .unwrap_or_else(|error| error.into_token_stream())
//! }
//!
//! fn derive_your_derive_inner(input: TokenStream) -> Result<TokenStream> {
//!     // Parse the struct or enum you want to implement a derive for
//!     let parse = Parse::new(input)?;
//!     // Get a reference to the generator
//!     let (mut generator, body) = parse.into_generator();
//!     match body {
//!         Body::Struct(body) => {
//!             // Implement your struct body here
//!             // See `Generator` for more information
//!             generator.impl_for("YourTrait")?
//!                     .generate_fn("your_fn")
//!                     .with_self_arg(FnSelfArg::RefSelf)
//!                     .body(|fn_body| {
//!                         fn_body.push_parsed("println!(\"Hello world\");");
//!                     })?;
//!         },
//!         Body::Enum(body) => {
//!             // Implement your enum body here
//!             // See `Generator` for more information
//!             generator.impl_for("YourTrait")?
//!                     .generate_fn("your_fn")
//!                     .with_self_arg(FnSelfArg::RefSelf)
//!                     .body(|fn_body| {
//!                         fn_body.push_parsed("println!(\"Hello world\");");
//!                     })?;
//!         },
//!     }
//!     generator.finish()
//! }
//! ```
//!
//! Will generate
//!
//! ```ignore
//! impl YourTrait for <Struct or Enum> {
//!     fn your_fn(&self) { // .generate_fn("your_fn").with_self_arg(FnSelfArg::RefSelf)
//!         println!("Hello world"); // fn_body.push_parsed(...)
//!     }
//! }
//! ```
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
