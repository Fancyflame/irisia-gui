use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_quote, Attribute, Ident};

use crate::derive_props::HandledField;

use super::GenHelper;

pub(super) fn impl_miscellaneous(helper: &GenHelper) -> TokenStream {
    let mut stream = make_struct(helper);
    stream.extend(impl_default(helper));
    stream.extend(regenerate_origin_struct(helper));
    stream.extend(set_props(helper));
    stream.extend(set_std_style_input(helper));
    stream
}

fn make_struct(helper: &GenHelper) -> TokenStream {
    let target_struct = &helper.target_struct;

    let generics_iter1 = helper.generics_iter();
    let generics_iter2 = helper.generics_iter();
    let fields = helper.fields.iter().map(|f| f.ident);

    quote! {
        struct #target_struct<#(#generics_iter1 = (),)*> {
            #(#fields: #generics_iter2,)*
        }
    }
}

fn impl_default(helper: &GenHelper) -> TokenStream {
    let fields = helper.fields.iter().map(|f| f.ident);

    let target_struct = &helper.target_struct;
    quote! {
        impl ::std::default::Default for #target_struct {
            fn default() -> Self {
                Self {
                    #(#fields: (),)*
                }
            }
        }
    }
}

fn regenerate_origin_struct(helper: &GenHelper) -> TokenStream {
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

fn set_props(helper: &GenHelper) -> TokenStream {
    let body = helper.fields.iter().enumerate().filter_map(
        |(
            index,
            HandledField {
                ident: fn_name,
                attr,
                ..
            },
        )| {
            // std style input cannot be setted as a common prop
            if attr.is_std_style_input() {
                return None;
            }

            let new_prop_type = quote!(NewPropType__);

            // field assignment
            let field_kv = helper.fields.iter().map(|f| {
                let field_name = f.ident;
                if field_name == *fn_name {
                    quote!(#field_name: (value,))
                } else {
                    quote!(#field_name: self.#field_name)
                }
            });

            // iter of generics at return position
            let ret_generics = helper.generics_iter().enumerate().map(|(index2, generic)| {
                if index2 == index {
                    quote!((#new_prop_type,))
                } else {
                    generic.into_token_stream()
                }
            });

            let GenHelper {
                target_struct, vis, ..
            } = &helper;

            // if renamed, replace function name with renamed value
            let fn_name = match &attr.options.rename {
                Some(renamed) => renamed,
                None => fn_name,
            };

            Some(quote! {
                #vis fn #fn_name<#new_prop_type>(self, value: #new_prop_type)
                    -> #target_struct<#(#ret_generics,)*>
                {
                    #target_struct {
                        #(#field_kv,)*
                    }
                }
            })
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

fn set_std_style_input(helper: &GenHelper) -> TokenStream {
    let GenHelper {
        target_struct,
        updater_generics,
        fields,
        ..
    } = helper;

    let style_generic: Ident = parse_quote!(__IrisiaStdInputStyles);

    let impl_generics = std::iter::once(&style_generic).chain(helper.generics_iter());

    let std_style_in_tuple = quote!((#style_generic,));
    let output_generics = fields
        .iter()
        .zip(helper.generics_iter())
        .map(|(field, generic)| {
            if field.attr.is_std_style_input() {
                &std_style_in_tuple as &dyn ToTokens
            } else {
                generic as _
            }
        });

    let init_fields = fields.iter().map(|field| {
        let name = field.ident;
        if field.attr.is_std_style_input() {
            quote!(#name: (__irisia_std_input_styles,))
        } else {
            quote!(#name: self.#name)
        }
    });

    quote! {
        impl<#(#impl_generics,)*> irisia::element::props::SetStdStyles<#style_generic>
            for #target_struct #updater_generics
        where
            #style_generic: irisia::style::StyleContainer
        {
            type Output = #target_struct<#(#output_generics,)*>;

            fn set_std_styles(
                self,
                __irisia_std_input_styles: &#style_generic
            ) -> Self::Output {
                #target_struct {
                    #(#init_fields,)*
                }
            }
        }
    }
}
