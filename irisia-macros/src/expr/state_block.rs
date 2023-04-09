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
    Codegen, StateExpr, VisitUnit,
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
            stmts_to_tokens(tokens, &self.stmts);
        });
    }
}

impl<T: Codegen> VisitUnit<T> for StateBlock<T> {
    fn visit_unit<F>(&self, f: &mut F)
    where
        F: FnMut(&StateExpr<T>),
    {
        for stmt in &self.stmts {
            stmt.visit_unit(f);
        }
    }

    fn visit_unit_mut<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut StateExpr<T>),
    {
        for stmt in &mut self.stmts {
            stmt.visit_unit_mut(f);
        }
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

pub fn stmts_to_tokens<T: Codegen>(tokens: &mut TokenStream, stmts: &[StateExpr<T>]) {
    T::empty(tokens);
    for expr in stmts {
        T::chain_applicate(tokens, |tokens| expr.to_tokens(tokens));
    }
}
