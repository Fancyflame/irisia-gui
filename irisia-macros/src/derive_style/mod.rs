use proc_macro2::{Span, TokenStream};
use syn::{Data, DeriveInput, Error, Member, Result};

mod parse_attr;
mod style_path;

pub fn derive_style(derive: DeriveInput) -> Result<TokenStream> {
    match &derive.data {
        Data::Struct(_) => derive_struct::derive_style_for_struct(derive),
        Data::Enum(_) => derive_enum::derive_style_for_enum(derive),
        Data::Union(_) => Err(Error::new(Span::call_site(), "union is unsupported")),
    }
}
