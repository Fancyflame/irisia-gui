use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    spanned::Spanned, Data, DataEnum, DataStruct, DeriveInput, Error, Fields, Ident, Result,
    Variant,
};

use self::to_tokens::write_stream;

mod attr_parse;
mod extract_paths;
mod parse_paths;
mod to_tokens;

pub fn derive_style(derive: DeriveInput) -> Result<TokenStream> {
    let item_span = derive.span();
    let DeriveInput {
        attrs,
        ident,
        data,
        generics,
        vis,
    } = derive;

    let mut tokens = {
        let (impl_gen, type_gen, where_clause) = generics.split_for_impl();
        quote! {
            #[automatically_derived]
            impl #impl_gen irisia::Style for #ident #type_gen
            #where_clause
            {}
        }
    };

    let mut write = |fields: &Fields, variant: Option<&Ident>, result| {
        write_stream(
            &mut tokens,
            fields,
            variant,
            &vis,
            &ident,
            &generics,
            result,
        )
    };

    match data {
        Data::Union(_) => return Err(Error::new(item_span, "union is unspported")),

        Data::Struct(DataStruct { fields, .. }) => {
            let result = extract_paths::analyze_fields(attrs, &fields)?;
            write(&fields, None, result);
        }

        Data::Enum(DataEnum { variants, .. }) => {
            if !extract_paths::get_attrs(&attrs)?.is_empty() {
                return Err(Error::new(
                    Span::call_site(),
                    "item-level irisia-macro is invalid for enum",
                ));
            }

            for Variant {
                attrs,
                ident,
                fields,
                ..
            } in variants
            {
                let result = extract_paths::analyze_fields(attrs, &fields)?;
                write(&fields, Some(&ident), result);
            }
        }
    }

    Ok(tokens)
}
