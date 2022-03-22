
// TODO:
// - Add documentation
// - Add more tests
// - Standardized user facing API.

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
    pub(crate) fn take(input: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Self {
        let visibility = Visibility::take(input);
        let is_async = consume_ident_if(input, "async").is_some();
        let is_unsafe = consume_ident_if(input, "unsafe").is_some();

        // Ignore everything until `fn` keyword
        let _ = input.skip_while(|tt| tt.to_string() != "fn").next();

        let name = consume_ident(input)
            .expect("Expected: `fn <name>`")
            .to_string();

        let generics = Generics::try_take(input).unwrap();

        Self {
            visibility,
            is_async,
            is_unsafe,
            name,
            generics,

            // For debugging purposes
            rest: input.collect(),
        }
    }
}

#[cfg(test)]
macro_rules! info { [$($t:tt)*] => { Function::take(&mut crate::token_stream(stringify!($($t)*))) }; }

#[test]
#[cfg(test)]
fn playground() {
    let foo = info! {
        pub fn foo<'a, T: Default>(arg: &'a T) -> () {}
    };
    println!("{:#?}", foo);
}

#[test]
#[cfg(test)]
fn test_simple() {
    let func = info! {
        pub async unsafe fn foo() {}
    };
    assert_eq!(func.visibility, Visibility::Pub);
    assert!(func.is_async);
    assert!(func.is_unsafe);
    assert_eq!(func.name, "foo");
    assert!(func.generics.is_none());

    // -------------------------------------------

    let func = info! {
        pub fn foo() {}
    };
    assert_eq!(func.visibility, Visibility::Pub);
    assert_eq!(func.name, "foo");

    // -------------------------------------------

    let func = info! {
        extern "C" fn bar() {}
    };
    assert_eq!(func.visibility, Visibility::Default);
    assert_eq!(func.name, "bar");
}
