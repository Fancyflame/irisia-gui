use proc_macro2::{TokenStream, TokenTree};
use quote::TokenStreamExt;
use syn::{
    parse::{ParseStream, Peek},
    Expr,
};

pub fn parse_maybe_incomplete_expr(input: ParseStream, until: impl Peek) -> Expr {
    let mut start = input.cursor();
    if let Ok(expr) = input.parse::<Expr>() {
        if input.peek(until) || input.is_empty() {
            return expr;
        }
    }

    let end = input.cursor();
    let mut tokens = TokenStream::new();
    while start < end {
        let (tt, cursor) = start.token_tree().unwrap();
        tokens.append(tt);
        start = cursor;
    }

    while !input.peek(until) && !input.is_empty() {
        tokens.append(input.parse::<TokenTree>().unwrap());
    }

    Expr::Verbatim(tokens)
}
