use super::utils::*;
use crate::prelude::TokenTree;
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
    pub(crate) fn take(input: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Self {
        consume_ident_if(input, "pub")
            .map(|_| {
                // check if the next token is `pub(...)`
                if let Some(TokenTree::Group(_)) = input.peek() {
                    // we just consume the visibility, we're not actually using it for generation
                    let _ = input.next();
                }
                Visibility::Pub
            })
            .unwrap_or(Visibility::Default)
    }
}

#[test]
fn test_visibility_take() {
    use crate::token_stream;

    assert_eq!(
        Visibility::Default,
        Visibility::take(&mut token_stream(""))
    );
    assert_eq!(
        Visibility::Pub,
        Visibility::take(&mut token_stream("pub"))
    );
    assert_eq!(
        Visibility::Pub,
        Visibility::take(&mut token_stream(" pub "))
    );
    assert_eq!(
        Visibility::Pub,
        Visibility::take(&mut token_stream("\tpub\t"))
    );
    assert_eq!(
        Visibility::Pub,
        Visibility::take(&mut token_stream("pub(crate)"))
    );
    assert_eq!(
        Visibility::Pub,
        Visibility::take(&mut token_stream(" pub ( crate ) "))
    );
    assert_eq!(
        Visibility::Pub,
        Visibility::take(&mut token_stream("\tpub\t(\tcrate\t)\t"))
    );

    assert_eq!(
        Visibility::Default,
        Visibility::take(&mut token_stream("pb"))
    );
}
