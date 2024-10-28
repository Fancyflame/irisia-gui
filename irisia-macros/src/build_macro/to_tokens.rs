use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::Parser, parse_quote, Expr, Pat};

use super::{pat_bind::PatBinds, Environment};

impl Environment {
    pub(super) fn cond_to_tokens(
        &self,
        wire_cond: impl ToTokens,
        if_true: impl ToTokens,
        if_false: impl ToTokens,
    ) -> TokenStream {
        quote! {
            ::irisia::structure::conditional(
                #wire_cond,
                #if_true,
                #if_false,
            )
        }
    }

    pub(super) fn pat_match_to_tokens(
        &self,
        source: &Expr,
        pat_binds: &PatBinds,
        if_matched: impl ToTokens,
        or_else: impl ToTokens,
    ) -> TokenStream {
        let PatBinds {
            pattern,
            guard,
            binds,
            ..
        } = pat_binds;

        let cond_fn = {
            let if_guard = guard.as_ref().map(|guard| quote! {if #guard});
            quote! {
                |__irisia_cond: &_| match __irisia_cond {
                    #[allow(unused_variables)]
                    #pattern #if_guard => ::std::option::Option::Some(
                        (#(#binds.clone(),)*)
                    ),
                    _ => ::std::option::Option::None
                }
            }
        };

        let if_matched = {
            let env = self.clone_env_wires();
            let temp_var = parse_quote!(__irisia_tuple_wire);

            let bind_vars = pat_binds.bind_var_from_wire(
                &temp_var,
                &Pat::parse_multi_with_leading_vert
                    .parse2(pat_binds.tuple_expr.clone())
                    .unwrap(),
            );

            quote! {
                {
                    #env
                    move |#temp_var| {
                        #bind_vars
                        #if_matched
                    }
                }
            }
        };

        quote! {
            ::irisia::structure::pat_match(
                #source,
                #cond_fn,
                #if_matched,
                #or_else,
            )
        }
    }

    pub(super) fn repeat_to_tokens(
        &self,
        pat_binds: &PatBinds,
        iter: &Expr,
        body: impl ToTokens,
    ) -> TokenStream {
        let env_wires = self.clone_env_wires();
        let var_binds =
            pat_binds.bind_var_from_wire(&parse_quote!(__irisia_input_wire), &pat_binds.pattern);

        quote! {
            ::irisia::structure::repeat(
                #iter,
                {
                    #env_wires
                    move |__irisia_input_wire| {
                        #var_binds
                        #body
                    }
                }
            )
        }
    }
}
