//! # Virtue, a sinless derive macro helper
//! 
//! ## Usage
//! 
//! ```ignore
//! #[proc_macro_derive(YourDerive, attributes(some, attributes, go, here))]
//! pub fn derive_encode(input: TokenStream) -> TokenStream {
//! }
//! ```
#![warn(missing_docs)]


mod error;
pub mod generate;
pub mod parse;

pub type Result<T = ()> = std::result::Result<T, Error>;
pub use self::error::Error;

#[cfg(test)]
pub mod prelude {
    pub use proc_macro2::*;
    pub use crate::Result;
}
#[cfg(not(test))]
pub mod prelude {
    extern crate proc_macro;

    pub use proc_macro::*;
    pub use crate::Result;
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
