use quote::{quote, ToTokens};
use syn::{parse::Parse, Expr, Pat, Token};

use crate::expr::{state_block::StateBlock, Codegen};
/*

*/
pub struct StateForLoop<T: Codegen> {
    pat: Pat,
    iter: Expr,
    key: Option<Expr>,
    body: StateBlock<T>,
}

impl<T: Codegen> Parse for StateForLoop<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![for]>()?;
        let pat = input.parse()?;
        input.parse::<Token![in]>()?;
        let iter = input.parse()?;
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
                ::cream_core::__macro_helper::__for_loop_iter_item_as_key(#iter, |#pat| #body)
            },
        };

        T::repetitive_applicate(tokens, |tokens| {
            iter.to_tokens(tokens);
        });
    }
}
