use proc_macro2::Ident;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    Error, Expr, Result, Token, Type,
};

use super::Codegen;

pub enum StateCommand<T: Codegen> {
    Extend {
        type_annotation: Option<Type>,
        value: Expr,
    },
    Key(Expr),
    Custom(T::Command),
}

impl<T: Codegen> Parse for StateCommand<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![@]>()?;
        let cmd_ident: Ident = input.parse()?;

        let cmd = match &*cmd_ident.to_string() {
            "extend" => parse_extend(input)?,
            "key" => parse_key(input)?,
            other => match T::parse_command(&other, input)? {
                Some(c) => StateCommand::Custom(c),
                None => {
                    return Err(Error::new(
                        cmd_ident.span(),
                        format!("unknown command `{other}`"),
                    ))
                }
            },
        };

        input.parse::<Token![;]>()?;
        Ok(cmd)
    }
}

impl<T: Codegen> ToTokens for StateCommand<T> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            StateCommand::Extend { value, .. } => value.to_tokens(tokens),
            StateCommand::Key(_) => {}
            StateCommand::Custom(other) => other.to_tokens(tokens),
        }
    }
}

fn parse_extend<T: Codegen>(input: ParseStream) -> Result<StateCommand<T>> {
    let c = if input.peek(Token![:]) {
        StateCommand::Extend {
            type_annotation: {
                input.parse::<Token![:]>()?;
                Some(input.parse()?)
            },
            value: {
                input.parse::<Token![=]>()?;
                input.parse()?
            },
        }
    } else {
        StateCommand::Extend {
            type_annotation: None,
            value: input.parse()?,
        }
    };
    Ok(c)
}

fn parse_key<T: Codegen>(input: ParseStream) -> Result<StateCommand<T>> {
    Ok(StateCommand::Key(input.parse()?))
}
