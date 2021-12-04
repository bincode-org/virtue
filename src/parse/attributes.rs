use super::utils::*;
use crate::prelude::{Delimiter, Group, Punct, TokenTree};
use crate::{Error, Result};
use std::iter::Peekable;

/// An attribute for the given struct, enum, field, etc
#[derive(Debug)]
#[non_exhaustive]
pub struct Attribute {
    /// The location this attribute was parsed at
    pub location: AttributeLocation,
    /// The punct token of the attribute. This will always be `Punct('#')`
    pub punct: Punct,
    /// The group of tokens of the attribute. You can parse this to get your custom attributes.
    pub tokens: Group,
}

/// The location an attribute can be found at
#[derive(PartialEq, Eq, Debug, Hash, Copy, Clone)]
#[non_exhaustive]
pub enum AttributeLocation {
    /// The attribute is on a container, which will be either a `struct` or an `enum`
    Container,
    /// The attribute is on an enum variant
    Variant,
    /// The attribute is on a field, which can either be a struct field or an enum variant field
    /// ```ignore
    /// struct Foo {
    ///     #[attr] // here
    ///     pub a: u8
    /// }
    /// struct Bar {
    ///     Baz {
    ///         #[attr] // or here
    ///         a: u8
    ///     }
    /// }
    /// ```
    Field,
}

impl Attribute {
    pub(crate) fn try_take(
        location: AttributeLocation,
        input: &mut Peekable<impl Iterator<Item = TokenTree>>,
    ) -> Result<Vec<Self>> {
        let mut result = Vec::new();

        while let Some(punct) = consume_punct_if(input, '#') {
            match input.peek() {
                Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Bracket => {
                    let group = assume_group(input.next());
                    result.push(Attribute {
                        location,
                        punct,
                        tokens: group,
                    });
                }
                Some(TokenTree::Group(g)) => {
                    return Err(Error::InvalidRustSyntax {
                        span: g.span(),
                        expected: format!("[] bracket, got {:?}", g.delimiter()),
                    });
                }
                Some(TokenTree::Punct(p)) if p.as_char() == '#' => {
                    // sometimes with empty lines of doc comments, we get two #'s in a row
                    // Just ignore this
                }
                token => return Error::wrong_token(token, "[] group or next # attribute"),
            }
        }
        Ok(result)
    }
}

#[test]
fn test_attributes_try_take() {
    use crate::token_stream;

    let stream = &mut token_stream("struct Foo;");
    assert!(Attribute::try_take(AttributeLocation::Container, stream)
        .unwrap()
        .is_empty());
    match stream.next().unwrap() {
        TokenTree::Ident(i) => assert_eq!(i, "struct"),
        x => panic!("Expected ident, found {:?}", x),
    }

    let stream = &mut token_stream("#[cfg(test)] struct Foo;");
    assert!(!Attribute::try_take(AttributeLocation::Container, stream)
        .unwrap()
        .is_empty());
    match stream.next().unwrap() {
        TokenTree::Ident(i) => assert_eq!(i, "struct"),
        x => panic!("Expected ident, found {:?}", x),
    }
}

/// Helper trait for functions like:
/// - [`IdentOrIndex::has_field_attribute`]
///
/// This can be implemented on your own type to make parsing easier.
///
/// [`IdentOrIndex::has_field_attribute`]: enum.IdentOrIndex.html#has_field_attribute
pub trait FromAttribute: Sized {
    /// Try to parse the given group into your own type. Return `None` if the parsing failed or if the attribute was not this type.
    fn parse(group: &Group) -> Option<Self>;
}
