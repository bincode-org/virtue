//! Utility functions
use crate::prelude::*;

/// Parse a tagged attribute. This is very helpful for implementing [`FromAttribute`].
///
/// A tagged attribute is an attribute in the form of `#[prefix(result)]`. This function will return `Some(result)` if the `prefix` matches.
///
/// # Examples
/// ```no_run
/// # use virtue::prelude::*;
/// # use virtue::utils::parse_tagged_attribute;
/// # use std::str::FromStr;
/// # fn parse_token_stream_group(input: &'static str) -> Group {
/// #     let token_stream: TokenStream = proc_macro2::TokenStream::from_str(input).unwrap().into();
/// #     match token_stream.into_iter().next() {
/// #         Some(TokenTree::Group(group)) => group,
/// #         _ => unreachable!(),
/// #     }
/// # }
/// // The attribute being parsed
/// let group: Group = parse_token_stream_group("#[prefix(result)]");
///
/// let inner = parse_tagged_attribute(&group, "prefix").unwrap();
///
/// // The stream will contain the contents of the `prefix(...)`
/// match inner.into_iter().next() {
///     Some(TokenTree::Ident(i)) if i.to_string() == "result" => {},
///     x => panic!("Unexpected token: {:?}", x)
/// }
/// ```
pub fn parse_tagged_attribute(group: &Group, prefix: &str) -> Option<TokenStream> {
    let stream = &mut group.stream().into_iter();
    if let Some(TokenTree::Ident(attribute_ident)) = stream.next() {
        if attribute_ident.to_string() != prefix {
            return None;
        }
        if let Some(TokenTree::Group(group)) = stream.next() {
            Some(group.stream())
        } else {
            None
        }
    } else {
        None
    }
}
