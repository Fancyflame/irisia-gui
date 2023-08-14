use quote::{quote, ToTokens};
use syn::{parse::Parse, Expr, Pat, Result, Token};

use crate::expr::{state_block::StateBlock, Codegen, StateExpr, VisitUnit};

pub struct StateForLoop<T: Codegen> {
    pat: Pat,
    iter: Expr,
    key: Option<Expr>,
    body: StateBlock<T>,
}

impl<T: Codegen> Parse for StateForLoop<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![for]>()?;
        let pat = Pat::parse_multi_with_leading_vert(input)?;
        input.parse::<Token![in]>()?;
        let iter = Expr::parse_without_eager_brace(input)?;
        let body: StateBlock<T> = input.parse()?;
        let key = body.get_key()?.cloned();

        Ok(StateForLoop {
            pat,
            iter,
            key,
            body,
        })
    }
}

impl<T: Codegen> ToTokens for StateForLoop<T> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let StateForLoop {
            pat,
            iter,
            body,
            key,
        } = self;

        let iter = match key {
            Some(k) => quote! {
                ::std::iter::Iterator::map(
                    #iter,
                    |#pat| (#k, #body)
                )
            },
            None => quote! {
                irisia::__for_loop_iter_item_as_key(#iter, |#pat| #body)
            },
        };

        tokens.extend(T::repetitive_applicate(iter));
    }
}

impl<T: Codegen> VisitUnit<T> for StateForLoop<T> {
    fn visit_unit<'a, F>(&'a self, depth: usize, f: &mut F) -> Result<()>
    where
        F: FnMut(&'a StateExpr<T>, usize) -> Result<()>,
        T: 'a,
    {
        self.body.visit_unit(depth, f)
    }

    fn visit_unit_mut<'a, F>(&'a mut self, depth: usize, f: &mut F) -> Result<()>
    where
        F: FnMut(&'a mut StateExpr<T>, usize) -> Result<()>,
        T: 'a,
    {
        self.body.visit_unit_mut(depth, f)
    }
}
