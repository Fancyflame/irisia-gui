use proc_macro2::TokenStream;
use quote::quote;
use syn::{LitStr, Result, parse::ParseStream};

pub fn pname_inner(input: &str) -> TokenStream {
    let chars = input.chars();
    quote! {
        (#(irisia::PChar::<#chars>,)*)
    }
}
