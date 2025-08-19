use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{Expr, Ident};

use crate::{build_macro::ast::FieldDecoration, consts::*};

use super::{ComponentStmt, FieldAssignment, GenerationEnv};

impl GenerationEnv {
    pub(super) fn gen_component(
        &self,
        ComponentStmt {
            comp_type,
            fields: all_fields,
            body,
            child_data,
        }: &ComponentStmt,
    ) -> TokenStream {
        let mut fields: Vec<&FieldAssignment<Ident>> = Vec::from_iter(all_fields.iter());

        let mut _cache = None;
        if !body.is_empty() {
            fields.push(_cache.insert(FieldAssignment {
                name: format_ident!("children"),
                decoration: FieldDecoration::None,
                value: Expr::Verbatim(GenerationEnv {}.gen_rc_chained(&body)),
            }));
        };

        let defs_tuple = binary_fold(&fields, &|&fa| {
            let FieldAssignment {
                name,
                value,
                decoration,
            } = fa;

            match decoration {
                FieldDecoration::None => {
                    quote! {
                        #PATH_COMPONENT::definition::proxy_signal_helper::check_eq(#value).get()
                    }
                }
                FieldDecoration::DirectAssign => {
                    quote! {
                        #PATH_COMPONENT::definition::direct_assign_helper::type_infer(
                            |#comp_type { #name, .. }| #name
                        ).infer(#value)
                    }
                }
                FieldDecoration::Event => unimplemented!(),
            }
        });

        let names_tuple = binary_fold(&fields, &|fa| fa.name.to_token_stream());

        let prop_assignments = fields.iter().map(|fa| {
            let name = &fa.name;
            let value = match fa.decoration {
                FieldDecoration::None => {
                    quote! {
                        irisia::coerce_hook!(#name)
                    }
                }
                FieldDecoration::DirectAssign => {
                    quote! { #name }
                }
                FieldDecoration::Event => unimplemented!(),
            };

            quote! {
                #name: #PATH_OPTION::Some(#value),
            }
        });

        let create_fn = quote! {
            |#names_tuple| {
                #comp_type {
                    #(#prop_assignments)*
                    ..#TRAIT_DEFAULT::default()
                }
            }
        };

        let append_child_data = child_data.as_ref().map(|child_data| {
            quote! {
                .set_child_data(#child_data)
            }
        });

        quote! {
            (
                #PATH_COMPONENT::UseComponent::new(
                    #create_fn,
                    #defs_tuple,
                )
                #append_child_data
            )
        }
    }
}

fn binary_fold<T, F>(slice: &[T], for_each: &F) -> TokenStream
where
    F: Fn(&T) -> TokenStream,
{
    match slice {
        [] => quote! {()},
        [one] => for_each(one),
        _ => {
            let (a, b) = slice.split_at(slice.len() / 2);
            let a = binary_fold(a, for_each);
            let b = binary_fold(b, for_each);
            quote! {(#a, #b)}
        }
    }
}
