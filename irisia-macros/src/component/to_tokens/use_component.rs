use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{Expr, Ident};

use crate::component::{
    ComponentStmt, FieldAssignMethod, FieldAssignment,
    to_tokens::{PATH_COMPONENT, PATH_OPTION},
};

use super::GenerationEnv;

const_quote! {
    const DEFAULT_TRAIT = { ::std::default::Default };
}

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

        let mut _body_fa = None;
        if !body.is_empty() {
            fields.push(_body_fa.insert(FieldAssignment {
                name: format_ident!("children"),
                method: FieldAssignMethod::HostingSignal,
                value: Expr::Verbatim(GenerationEnv {}.gen_rc_chained(&body)),
            }));
        };

        let defs_tuple = field_asgn_binary_fold(&fields, &|&fa| {
            let FieldAssignment {
                name,
                value,
                method,
            } = fa;

            match method {
                FieldAssignMethod::HostingSignal => {
                    quote! {
                        #PATH_COMPONENT::proxy_signal_helper::check_eq(#value).get()
                    }
                }
                FieldAssignMethod::Direct => {
                    quote! {
                        #PATH_COMPONENT::direct_assign_helper::type_infer(
                            |#comp_type { #name, .. }| #name
                        ).infer(#value)
                    }
                }
            }
        });

        let names_tuple = field_asgn_binary_fold(&fields, &|fa| fa.name.to_token_stream());

        let prop_assignments = fields.iter().map(|fa| {
            let name = &fa.name;
            let value = match fa.method {
                FieldAssignMethod::HostingSignal => {
                    quote! {
                        irisia::coerce_hook!(#name)
                    }
                }
                FieldAssignMethod::Direct => {
                    quote! { #name }
                }
            };

            quote! {
                #name: #PATH_OPTION::Some(#value),
            }
        });

        let create_fn = quote! {
            |#names_tuple| {
                #comp_type {
                    #(#prop_assignments)*
                    ..#DEFAULT_TRAIT::default()
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

fn field_asgn_binary_fold<T, F>(slice: &[T], for_each: &F) -> TokenStream
where
    F: Fn(&T) -> TokenStream,
{
    match slice {
        [] => quote! {()},
        [one] => for_each(one),
        _ => {
            let (a, b) = slice.split_at(slice.len() / 2);
            let a = field_asgn_binary_fold(a, for_each);
            let b = field_asgn_binary_fold(b, for_each);
            quote! {(#a, #b)}
        }
    }
}
