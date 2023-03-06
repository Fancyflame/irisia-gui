use expr::{state_block::parse_stmts, state_block::StateBlock};
use proc_macro::TokenStream;
use quote::ToTokens;
use style::StyleCodegen;
use syn::parse::{ParseStream, Parser};

mod element;
pub(crate) mod expr;
mod style;

#[proc_macro]
pub fn build(input: TokenStream) -> TokenStream {
    let helper = |input: ParseStream| element::build::build(input, false);
    match helper.parse(input) {
        Ok(t) => t.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn render(input: TokenStream) -> TokenStream {
    let helper = |input: ParseStream| element::build::build(input, true);
    match helper.parse(input) {
        Ok(t) => t.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn style(input: TokenStream) -> TokenStream {
    let stmts = match parse_stmts::<StyleCodegen>.parse(input) {
        Ok(stmts) => stmts,
        Err(e) => return e.to_compile_error().into(),
    };

    let stream = StateBlock {
        brace: Default::default(),
        stmts,
    }
    .into_token_stream();
    stream.into()
}
