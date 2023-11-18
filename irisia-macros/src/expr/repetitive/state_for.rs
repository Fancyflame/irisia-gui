use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::Parse, Expr, Pat, Token};

use crate::expr::{state_block::StateBlock, CodegenAlias, StmtTree};

pub struct StateForLoop<T: StmtTree> {
    pub pat: Pat,
    pub iter: Expr,
    pub key: Option<Expr>,
    pub body: StateBlock<T>,
}

impl<T: StmtTree> Parse for StateForLoop<T> {
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

impl<T: CodegenAlias> StateForLoop<T> {
    pub(super) fn expr_iter(&self) -> TokenStream {
        let StateForLoop {
            pat,
            iter,
            body,
            key,
        } = self;

        match key {
            Some(k) => quote! {
                ::std::iter::Iterator::map(
                    #iter,
                    |#pat| (#k, #body)
                )
            },
            None => quote! {
                irisia::__private::for_loop_iter_item_as_key(#iter, |#pat| #body)
            },
        }
    }
}
