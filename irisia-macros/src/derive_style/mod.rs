use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Data, DataEnum, DataStruct, DeriveInput, Error, Result};

mod parse_attr;
mod style_path;

pub fn derive_style(
    DeriveInput {
        attrs,
        ident,
        generics,
        data,
        ..
    }: DeriveInput,
) -> Result<TokenStream> {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let mut tokens = quote! {
        impl #impl_generics ::irisia::Style
            for #ident #ty_generics #where_clause
        {}
    };

    match &data {
        Data::Struct(DataStruct { fields, .. }) => {
            parse_attr::derive_for(&attrs, &ident, None, &generics, fields)?.to_tokens(&mut tokens);
        }
        Data::Enum(DataEnum { variants, .. }) => {
            for variant in variants {
                tokens.extend(parse_attr::derive_for(
                    &variant.attrs,
                    &ident,
                    Some(&variant.ident),
                    &generics,
                    &variant.fields,
                )?);
            }
        }
        Data::Union(_) => return Err(Error::new(Span::call_site(), "union is unsupported")),
    }

    Ok(tokens)
}
