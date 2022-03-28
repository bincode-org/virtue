// TODO:
// - Add documentation
// - Add more tests
// - Standardize user facing API.

use super::utils::*;
use super::*;
use std::iter::Peekable;

#[derive(Debug)]
struct Function {
    visibility: Visibility,
    is_async: bool,
    is_unsafe: bool,

    name: String,
    generics: Option<Generics>,

    // TODO:
    // args: Vec<FnArg>,
    // where_cl : WhereClause,
    // ret_ty : ReturnType,
    // body: FnBody,
    #[allow(dead_code)]
    rest: TokenStream, // For debugging purposes
}

impl Function {
    pub(crate) fn try_take(input: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Result<Self> {
        let visibility = Visibility::take(input);
        let is_async = consume_ident_if_eq(input, "async").is_some();
        let is_unsafe = consume_ident_if_eq(input, "unsafe").is_some();

        // Ignore everything until `fn` keyword
        let _ = input.skip_while(|tt| tt.to_string() != "fn").next();

        let name = consume_ident(input)
            .ok_or(Error::ExpectedIdent(Span::call_site()))?
            .to_string();

        let generics = Generics::try_take(input)?;

        Ok(Self {
            visibility,
            is_async,
            is_unsafe,
            name,
            generics,

            // For debugging purposes
            rest: input.collect(),
        })
    }
}

#[cfg(test)]
macro_rules! token_stream { [$($t:tt)*] => { Function::try_take(&mut crate::token_stream(stringify!($($t)*))).unwrap() }; }

#[test]
#[cfg(test)]
fn playground() {
    let foo = token_stream! {
        pub fn foo<'a, 'b, T>(
            arg2: &'a [T],
            // &arg1: &'b u8,
            // mut arg3: String,
            // arg4: impl AsRef<[u8]>,
        ) -> ()
        where
            T: Default,
        {
            println!("{}", "Hello, world!");
        }
    };
    println!("{:#?}", foo);
}

#[test]
fn test_simple() {
    let func = token_stream! {
        pub async unsafe fn foo() {}
    };
    assert_eq!(func.visibility, Visibility::Pub);
    assert!(func.is_async);
    assert!(func.is_unsafe);
    assert_eq!(func.name, "foo");
    assert!(func.generics.is_none());

    // -------------------------------------------

    let func = token_stream! {
        pub fn foo() {}
    };
    assert_eq!(func.visibility, Visibility::Pub);
    assert_eq!(func.name, "foo");

    // -------------------------------------------

    let func = token_stream! {
        extern "C" fn bar() {}
    };
    assert_eq!(func.visibility, Visibility::Default);
    assert_eq!(func.name, "bar");
}
