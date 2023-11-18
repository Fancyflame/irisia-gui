use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::Parse, Expr, Token};

use crate::expr::{state_block::StateBlock, CodegenAlias, StmtTree};

pub struct StateWhile<T: StmtTree> {
    pub cond: Expr,
    pub key: Expr,
    pub state_block: StateBlock<T>,
}

impl<T: StmtTree> Parse for StateWhile<T> {
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

impl<T: CodegenAlias> StateWhile<T> {
    pub(super) fn expr_iter(&self) -> TokenStream {
        let StateWhile {
            cond,
            key,
            state_block,
        } = self;

        quote! {
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
        }
    }
}
