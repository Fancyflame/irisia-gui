use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    braced,
    parse::{Nothing, Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    token::Brace,
    Expr, Result, Token,
};

use super::{
    state_command::{StateCommand, StateCommandBody},
    Codegen, StateExpr,
};

// {Empty.xxx().xxx()}
pub struct StateBlock<T: Codegen> {
    pub brace: Brace,
    pub stmts: Vec<StateExpr<T>>,
}

impl<T: Codegen> Default for StateBlock<T> {
    fn default() -> Self {
        StateBlock {
            brace: Default::default(),
            stmts: Vec::new(),
        }
    }
}

impl<T: Codegen> Parse for StateBlock<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let states;
        let brace = braced!(states in input);
        Ok(StateBlock {
            brace,
            stmts: parse_stmts(&states)?,
        })
    }
}

impl<T: Codegen> ToTokens for StateBlock<T> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.brace.surround(tokens, |tokens| {
            tokens.extend(stmts_to_tokens(&self.stmts));
        });
    }
}

impl<T: Codegen> StateBlock<T> {
    pub fn get_key(&self) -> Result<Option<&Expr>> {
        let mut key = None;
        for stmt in &self.stmts {
            if let StateExpr::Command(StateCommand {
                body: StateCommandBody::Key(key_expr),
                ..
            }) = stmt
            {
                if key.replace(key_expr).is_some() {
                    return Err(syn::Error::new(
                        key_expr.span(),
                        "duplicated key declaration",
                    ));
                }
            }
        }
        Ok(key)
    }
}

pub fn parse_stmts<T: Codegen>(input: ParseStream) -> Result<Vec<StateExpr<T>>> {
    let vec = Punctuated::<StateExpr<T>, Nothing>::parse_terminated_with(input, |input| {
        let expr = input.parse()?;
        let _ = input.parse::<Option<Token![;]>>();
        Ok(expr)
    })?
    .into_iter()
    .collect();
    Ok(vec)
}

pub fn stmts_to_tokens<T: Codegen>(stmts: &[StateExpr<T>]) -> TokenStream {
    let mut tokens = T::empty();
    for expr in stmts {
        tokens = T::chain_applicate(tokens, expr);
    }
    tokens
}
