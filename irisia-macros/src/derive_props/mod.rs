use case::CaseExt;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_quote, punctuated::Punctuated, Data, DeriveInput, Error, Field, Fields, FieldsNamed,
    FieldsUnnamed, Generics, Ident, Result,
};

use self::field::FieldProps;

mod field;

struct Info {
    ident: Ident,
    generics: Generics,
    props: Vec<FieldProps>,
}

pub fn derive(input: DeriveInput) -> Result<TokenStream> {
    let fields = match input.data {
        Data::Struct(s) => s.fields,
        _ => return Err(Error::new_spanned(input, "only struct is supported")),
    };

    let props = match fields {
        Fields::Unnamed(f) => {
            return Err(Error::new_spanned(f, "fields are expected to have name"))
        }
        Fields::Named(FieldsNamed { named: fields, .. }) => {
            let mut vec = Vec::new();
            for f in fields {
                vec.push(FieldProps::parse(f)?);
            }
            vec
        }
        Fields::Unit => Vec::new(),
    };

    let info = Info {
        ident: input.ident,
        generics: input.generics,
        props,
    };

    todo!()
}

impl Info {
    fn make_props_struct(&self) -> TokenStream {
        let generics_vec: Vec<Ident> = self
            .props
            .iter()
            .enumerate()
            .map(|(index, f)| match &f.field.ident {
                Some(id) => format_ident!("TypeOf{}", id.to_string().to_camel()),
                None => format_ident!("TypeOf{}", index),
            })
            .collect();

        let ident = &self.ident;

        quote! {
            impl<#(#generics_vec),*> #ident<#(#generics_vec),*> {
                #(
                    pub fn
                )
            }
        }
    }
}
