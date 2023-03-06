use proc_macro2::{Ident, Span};
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    Error, Expr, Result, Token,
};

use super::Codegen;

pub struct StateCommand<T: Codegen> {
    pub span: Span,
    pub body: StateCommandBody<T>,
}

pub enum StateCommandBody<T: Codegen> {
    Extend(Expr),
    Key(Expr),
    Custom(T::Command),
}

impl<T: Codegen> Parse for StateCommand<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![@]>()?;
        let cmd_ident: Ident = input.parse()?;

        let cmd = match &*cmd_ident.to_string() {
            "extend" => StateCommandBody::Extend(input.parse()?),
            "key" => parse_key(input)?,
            other => match T::parse_command(&other, input)? {
                Some(c) => StateCommandBody::Custom(c),
                None => {
                    return Err(Error::new(
                        cmd_ident.span(),
                        format!("unknown command `{other}`"),
                    ))
                }
            },
        };
        input.parse::<Token![;]>()?;

        Ok(StateCommand {
            span: cmd_ident.span(),
            body: cmd,
        })
    }
}

impl<T: Codegen> ToTokens for StateCommand<T> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match &self.body {
            StateCommandBody::Extend(expr) => expr.to_tokens(tokens),
            StateCommandBody::Key(_) => T::empty(tokens),
            StateCommandBody::Custom(other) => other.to_tokens(tokens),
        }
    }
}

fn parse_key<T: Codegen>(input: ParseStream) -> Result<StateCommandBody<T>> {
    Ok(StateCommandBody::Key(input.parse()?))
}
