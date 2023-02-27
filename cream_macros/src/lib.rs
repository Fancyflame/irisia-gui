use expr::{state_block::parse_stmts, state_block::StateBlock};
use proc_macro::TokenStream;
use quote::ToTokens;
use style::StyleCodegen;
use syn::{parse::Parser, parse_macro_input};
use uninit_type::{set::SetTypeList, UninitTypeList};

mod element;
pub(crate) mod expr;
mod style;
mod uninit_type;

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
    //println!("{}", stream.to_string());
    stream.into()
}

#[proc_macro]
pub fn uninit_type(input: TokenStream) -> TokenStream {
    let result = parse_macro_input!(input as UninitTypeList)
        .0
        .into_iter()
        .map(|x| x.to_output());
    TokenStream::from_iter(result)
}

#[proc_macro]
pub fn set_type(input: TokenStream) -> TokenStream {
    let result = parse_macro_input!(input as SetTypeList)
        .0
        .into_iter()
        .map(|x| x.to_output());
    TokenStream::from_iter(result)
}
