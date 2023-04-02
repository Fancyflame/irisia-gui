use quote::{quote, ToTokens};
use syn::{parse::Parse, Expr, Token};

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

        T::repetitive_applicate(tokens, |tokens| {
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
            .to_tokens(tokens);
        });
    }
}

impl<T: Codegen> VisitUnit<T> for StateWhile<T> {
    fn visit_unit<F>(&self, f: &mut F)
    where
        F: FnMut(&StateExpr<T>),
    {
        self.state_block.visit_unit(f);
    }

    fn visit_unit_mut<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut StateExpr<T>),
    {
        self.state_block.visit_unit_mut(f);
    }
}
