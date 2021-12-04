use super::utils::*;
use crate::prelude::TokenTree;
use crate::Result;
use std::iter::Peekable;

/// The visibility of a struct, enum, field, etc
#[derive(Debug, PartialEq, Clone)]
pub enum Visibility {
    /// Default visibility. Most items are private by default.
    Default,

    /// Public visibility
    Pub,
}

impl Visibility {
    pub(crate) fn try_take(input: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Result<Self> {
        if let Some(TokenTree::Ident(ident)) = input.peek() {
            if ident_eq(ident, "pub") {
                // Consume this token
                assume_ident(input.next());

                // check if the next token is `pub(...)`
                if let Some(TokenTree::Group(_)) = input.peek() {
                    // we just consume the visibility, we're not actually using it for generation
                    assume_group(input.next());
                }

                return Ok(Visibility::Pub);
            }
        }
        Ok(Visibility::Default)
    }
}

#[test]
fn test_visibility_try_take() {
    use crate::token_stream;

    assert_eq!(
        Visibility::Default,
        Visibility::try_take(&mut token_stream("")).unwrap()
    );
    assert_eq!(
        Visibility::Pub,
        Visibility::try_take(&mut token_stream("pub")).unwrap()
    );
    assert_eq!(
        Visibility::Pub,
        Visibility::try_take(&mut token_stream(" pub ")).unwrap(),
    );
    assert_eq!(
        Visibility::Pub,
        Visibility::try_take(&mut token_stream("\tpub\t")).unwrap()
    );
    assert_eq!(
        Visibility::Pub,
        Visibility::try_take(&mut token_stream("pub(crate)")).unwrap()
    );
    assert_eq!(
        Visibility::Pub,
        Visibility::try_take(&mut token_stream(" pub ( crate ) ")).unwrap()
    );
    assert_eq!(
        Visibility::Pub,
        Visibility::try_take(&mut token_stream("\tpub\t(\tcrate\t)\t")).unwrap()
    );

    assert_eq!(
        Visibility::Default,
        Visibility::try_take(&mut token_stream("pb")).unwrap()
    );
}
