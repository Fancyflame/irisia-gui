use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::Attribute;

use super::GenHelper;

pub(super) fn make_struct(helper: &GenHelper) -> TokenStream {
    if helper.no_fields() {
        return quote! {
            struct Foo;
        };
    }

    let generics_iter1 = helper.generics_iter();
    let generics_iter2 = helper.generics_iter();
    let fields = helper.field_iter().map(|(ident, _)| ident);

    quote! {
        struct Foo<#(#generics_iter1 = (),)*> {
            #(#fields: #generics_iter2,)*
        }
    }
}

pub(super) fn impl_default(helper: &GenHelper) -> TokenStream {
    let self_body = if helper.no_fields() {
        None
    } else {
        let fields = helper.field_iter().map(|(ident, _)| ident);
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

    let mut tokens = TokenStream::new();
    for (index, (fn_name, _)) in helper.field_iter().enumerate() {
        let new_prop_type = quote!(NewPropType__);

        let field_kv = helper.field_iter().map(|(field_name, _)| {
            if field_name == fn_name {
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

        let struct_name = &helper.target_struct;

        tokens.extend(quote! {
            fn #fn_name<#new_prop_type>(self, value: #new_prop_type)
                -> #struct_name<#(#ret_generics,)*>
            {
                #struct_name {
                    #(#field_kv,)*
                }
            }
        });
    }

    helper.impl_self(tokens)
}
