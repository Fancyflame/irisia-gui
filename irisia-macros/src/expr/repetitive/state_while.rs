use quote::{quote, ToTokens};
use syn::{parse::Parse, Expr, Result, Token};

use crate::expr::{state_block::StateBlock, Codegen, StateExpr, VisitUnit};

pub struct StateWhile<T: Codegen> {
    cond: Expr,
    key: Expr,
    state_block: StateBlock<T>,
}

impl<T: Codegen> Parse for StateWhile<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![while]>()?;
        let cond = Expr::parse_without_eager_brace(input)?;
        let state_block: StateBlock<T> = input.parse()?;
        let key = state_block.get_key()?;

        match key {
            Some(k) => Ok(StateWhile {
                cond,
                key: k.clone(),
                state_block,
            }),
            None => Err(input.error("missing key declaration. consider add a `@key ...;` command")),
        }
    }
}

impl<T: Codegen> ToTokens for StateWhile<T> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let StateWhile {
            cond,
            key,
            state_block,
        } = self;

        tokens.extend(T::repetitive_applicate(quote! {
            ::std::iter::from_fn(
                || if #cond {
                    ::std::option::Option::Some((
                        #key,
                        #state_block
                    ))
                } else {
                    ::std::option::Option::None
                }
            )
        }));
    }
}

impl<T: Codegen> VisitUnit<T> for StateWhile<T> {
    fn visit_unit<'a, F>(&'a self, depth: usize, f: &mut F) -> Result<()>
    where
        F: FnMut(&'a StateExpr<T>, usize) -> Result<()>,
        T: 'a,
    {
        self.state_block.visit_unit(depth, f)
    }

    fn visit_unit_mut<'a, F>(&'a mut self, depth: usize, f: &mut F) -> Result<()>
    where
        F: FnMut(&'a mut StateExpr<T>, usize) -> Result<()>,
        T: 'a,
    {
        self.state_block.visit_unit_mut(depth, f)
    }
}
