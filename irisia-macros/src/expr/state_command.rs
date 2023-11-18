use proc_macro2::{Ident, Span};
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    Error, Expr, Result, Token,
};

use super::{CodegenAlias, StmtTree};

pub struct StateCommand<T: StmtTree> {
    pub span: Span,
    pub body: StateCommandBody<T>,
}

pub enum StateCommandBody<T: StmtTree> {
    Key(Expr),
    Custom(T::Command),
}

impl<T: StmtTree> Parse for StateCommand<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![@]>()?;
        let cmd_ident: Ident = input.parse()?;

        let cmd = match &*cmd_ident.to_string() {
            "key" => parse_key(input)?,
            other => match T::parse_command(other, input)? {
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

impl<T: CodegenAlias> ToTokens for StateCommand<T> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if let StateCommandBody::Custom(other) = &self.body {
            if let Some(t) = T::command_applicate(other) {
                tokens.extend(t);
                return;
            }
        }

        tokens.extend(T::empty());
    }
}

fn parse_key<T: StmtTree>(input: ParseStream) -> Result<StateCommandBody<T>> {
    Ok(StateCommandBody::Key(input.parse()?))
}
