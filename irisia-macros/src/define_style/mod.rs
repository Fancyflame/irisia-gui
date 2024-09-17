use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Data, DataEnum, DataStruct, DeriveInput, Error, Result};

mod define;

pub use define::parse as define;

pub fn derive_style(
    DeriveInput {
        attrs,
        ident,
        generics,
        data,
        ..
    }: DeriveInput,
) -> Result<TokenStream> {
    todo!()
}
