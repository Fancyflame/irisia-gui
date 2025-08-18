use ast::Stmt;
use proc_macro2::TokenStream;
use syn::parse::{Parse, ParseStream};

mod ast;
mod parse;
mod to_tokens;

pub fn build_macro(input: ParseStream) -> syn::Result<TokenStream> {
    parse::parse_stmts(input).map(|stmts| to_tokens::gen_code(&stmts))
}
