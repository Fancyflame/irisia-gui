use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_quote, Expr};

use super::{pat_bind::PatBinds, ElementDeclaration, Environment};

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
        origin: &Expr,
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

        let if_guard = guard.as_ref().map(|guard| quote! {if #guard});

        let wire_option = self.create_wire(&parse_quote! {
            match &(#origin) {
                #[allow(unused_variables)]
                #pattern #if_guard => ::std::option::Option::Some(
                    (#(::std::clone::Clone::clone(#binds),)*)
                ),
                _ => ::std::option::Option::None
            }
        });

        let env = self.env_to_tokens();
        let bind_vars =
            pat_binds.bind_var_from_wire(&parse_quote!(__irisia_tuple_wire), &pat_binds.tuple_expr);
        let if_matched = quote! {
            {
                #env
                move |__irisia_tuple_wire| {
                    #bind_vars
                    #if_matched
                }
            }
        };

        quote! {
            ::irisia::structure::pat_match(
                #wire_option,
                #if_matched,
                #or_else,
            )
        }
    }

    pub(super) fn repeat_to_tokens(
        &self,
        iter: &Expr,
        key: &Expr,
        pat_binds: &PatBinds,
        body: impl ToTokens,
    ) -> TokenStream {
        let env = self.env_to_tokens();
        let var_binds =
            pat_binds.bind_var_from_wire(&parse_quote!(__irisia_wire), &pat_binds.pattern);
        let key_pattern = &pat_binds.pattern;

        quote! {
            ::irisia::structure::repeat({
                #env
                move |__irisia_mutator| __irisia_mutator.update(
                    #iter,
                    |#[allow(unused_variables)] #key_pattern| #key,
                    |__irisia_wire| {
                        #var_binds
                        #body
                    }
                )
            })
        }
    }

    pub(super) fn element_to_tokens(
        &self,
        ElementDeclaration {
            el_type,
            wired_props,
            styles,
            slot,
            on_create,
        }: &ElementDeclaration,
    ) -> TokenStream {
        let props = wired_props.iter().map(|(key, value)| {
            quote! {
                #key: ::std::convert::From::from(#value),
            }
        });

        let env = self.env_to_tokens();

        quote! {
            ::irisia::structure::single::<#el_type>(
                ::irisia::element::ElementPropsAlias::<#el_type> {
                    #(#props)*
                    ..::std::default::Default::default()
                },
                #styles,
                #slot,
                {
                    #env
                    #on_create
                },
            )
        }
    }
}
