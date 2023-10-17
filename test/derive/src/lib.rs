use virtue::prelude::*;

#[proc_macro_derive(RetHi)]
pub fn derive_ret_hi(input: TokenStream) -> TokenStream {
    derive_ret_hi_inner(input).unwrap_or_else(|error| error.into_token_stream())
}

fn derive_ret_hi_inner(input: TokenStream) -> Result<TokenStream> {
    let parse = Parse::new(input)?;
    let (mut generator, _, _) = parse.into_generator();
    generator
        .generate_impl()
        .generate_fn("hi")
        .with_attr("inline(never)")
        .with_self_arg(FnSelfArg::RefSelf)
        .with_return_type("&'static str")
        .body(|body| {
            body.lit_str("hi");
            Ok(())
        })?;
    generator.finish()
}
