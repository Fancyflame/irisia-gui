use proc_macro2::{Span, TokenStream};
use syn::{Data, DeriveInput, Error, Member, Result};

mod attributes;
mod codegen_utils;
mod derive_enum;
mod derive_struct;
mod variant_analyzer;

pub fn derive_style(derive: DeriveInput) -> Result<TokenStream> {
    match &derive.data {
        Data::Struct(_) => derive_struct::derive_style_for_struct(derive),
        Data::Enum(_) => derive_enum::derive_style_for_enum(derive),
        Data::Union(_) => Err(Error::new(Span::call_site(), "union is unsupported")),
    }
}

pub fn tag_to_string(tag: &Member) -> String {
    match tag {
        Member::Named(id) => id.to_string(),
        Member::Unnamed(index) => index.index.to_string(),
    }
}
