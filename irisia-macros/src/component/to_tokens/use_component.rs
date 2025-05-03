use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{Expr, Path};

use crate::component::{
    ComponentStmt, FieldAssignMethod, FieldAssignment,
    to_tokens::{PATH_COMPONENT, PATH_OPTION},
};

use super::GenerationEnv;

const_quote! {
    const DEFAULT_TRAIT = { ::std::default::Default };
}

impl GenerationEnv<'_> {
    pub(super) fn gen_component(
        &self,
        ComponentStmt {
            comp_type,
            fields: all_fields,
            body,
        }: &ComponentStmt,
    ) -> TokenStream {
        let mut fields: Vec<&FieldAssignment> = Vec::with_capacity(all_fields.capacity());
        let mut parent_prop_fields: Vec<&FieldAssignment> = Vec::new();
        for field in all_fields {
            if let FieldAssignMethod::ParentProp = field.method {
                parent_prop_fields.push(field);
            } else {
                fields.push(field);
            }
        }

        let mut _body_fa = None;
        if !body.is_empty() {
            fields.push(
                _body_fa.insert(FieldAssignment {
                    name: format_ident!("children"),
                    method: FieldAssignMethod::HostingSignal,
                    value: Expr::Verbatim(
                        GenerationEnv {
                            parent_component: Some(comp_type),
                        }
                        .gen_rc_chained(&body),
                    ),
                }),
            );
        };

        let defs = field_asgn_binary_fold(&fields, &|fa| {
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
                            |v: #comp_type| v.#name
                        ).infer(#value)
                    }
                }
                FieldAssignMethod::ParentProp => unreachable!(),
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
                FieldAssignMethod::ParentProp => unreachable!(),
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

        let parent_prop_expr =
            gen_parent_prop_expr(parent_prop_fields.iter().copied(), self.parent_component);

        quote! {
            {
                #PATH_COMPONENT::UseComponent::<#comp_type, _, _, _>::new(
                    #parent_prop_expr,
                    #create_fn,
                    #defs,
                )
            }
        }
    }
}

pub(super) fn gen_parent_prop_expr<'a, I>(
    parent_prop_fields: I,
    parent_type: Option<&Path>,
) -> TokenStream
where
    I: Iterator<Item = &'a FieldAssignment> + ExactSizeIterator + Clone,
{
    // let Some(parent_type) = parent_type else {
    //     assert_eq!(parent_prop_fields.len(), 0);
    //     return quote! {()};
    // };

    if parent_prop_fields.len() == 0 {
        return match parent_type {
            Some(parent_type) => quote! {
                <#PATH_COMPONENT::GetChildProps::<#parent_type> as #DEFAULT_TRAIT>::default()
            },
            None => quote! {
                #DEFAULT_TRAIT::default()
            },
        };
    }

    let names = parent_prop_fields.clone().map(|fa| &fa.name);
    let values = parent_prop_fields.map(|fa| &fa.value);

    let parent_type = parent_type.expect("parent type must be provided as parent-property defined");
    quote! {
        #PATH_COMPONENT::GetChildProps::<#parent_type> {
            #(#names: #values,)*
            ..#DEFAULT_TRAIT::default()
        }
    }
}

fn field_asgn_binary_fold<F>(slice: &[&FieldAssignment], for_each: &F) -> TokenStream
where
    F: Fn(&FieldAssignment) -> TokenStream,
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
