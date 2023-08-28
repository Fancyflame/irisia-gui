use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::Attribute;

use crate::derive_props::HandledField;

use super::GenHelper;

pub(super) fn make_struct(helper: &GenHelper) -> TokenStream {
    let target_struct = &helper.target_struct;

    if helper.no_fields() {
        return quote! {
            struct #target_struct;
        };
    }

    let generics_iter1 = helper.generics_iter();
    let generics_iter2 = helper.generics_iter();
    let fields = helper.fields.iter().map(|f| f.ident);

    quote! {
        struct #target_struct<#(#generics_iter1 = (),)*> {
            #(#fields: #generics_iter2,)*
        }
    }
}

pub(super) fn impl_default(helper: &GenHelper) -> TokenStream {
    let self_body = if helper.no_fields() {
        None
    } else {
        let fields = helper.fields.iter().map(|f| f.ident);
        Some(quote! {
            {
                #(#fields: (),)*
            }
        })
    };

    let target_struct = &helper.target_struct;
    quote! {
        impl ::std::default::Default for #target_struct {
            fn default() -> Self {
                Self #self_body
            }
        }
    }
}

pub(super) fn regenerate_origin_struct(helper: &GenHelper) -> TokenStream {
    fn clear_attrs(attrs: &mut Vec<Attribute>) {
        attrs.retain(|attr| !attr.meta.path().is_ident("props"))
    }

    let mut stru = helper.item.clone();
    clear_attrs(&mut stru.attrs);
    for field in stru.fields.iter_mut() {
        clear_attrs(&mut field.attrs);
    }

    stru.into_token_stream()
}

pub(super) fn set_props(helper: &GenHelper) -> TokenStream {
    if helper.no_fields() {
        return quote!();
    }

    let body = helper.fields.iter().enumerate().map(
        |(
            index,
            HandledField {
                ident: fn_name,
                attr,
                ..
            },
        )| {
            let new_prop_type = quote!(NewPropType__);

            let field_kv = helper.fields.iter().map(|f| {
                let field_name = f.ident;
                if field_name == *fn_name {
                    quote!(#field_name: (value,))
                } else {
                    quote!(#field_name: self.#field_name)
                }
            });

            let ret_generics = helper.generics_iter().enumerate().map(|(index2, gen)| {
                if index2 == index {
                    quote!((#new_prop_type,))
                } else {
                    gen.into_token_stream()
                }
            });

            let GenHelper {
                target_struct, vis, ..
            } = &helper;

            let fn_name = match &attr.options.rename {
                Some(renamed) => renamed,
                None => fn_name,
            };

            quote! {
                #vis fn #fn_name<#new_prop_type>(self, value: #new_prop_type)
                    -> #target_struct<#(#ret_generics,)*>
                {
                    #target_struct {
                        #(#field_kv,)*
                    }
                }
            }
        },
    );

    let GenHelper {
        target_struct,
        updater_generics,
        ..
    } = helper;

    quote! {
        impl #updater_generics #target_struct #updater_generics {
            #(#body)*
        }
    }
}
