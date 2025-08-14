use proc_macro2::TokenStream;
use quote::quote;
use syn::{LitStr, Result, parse::ParseStream};

pub fn pname(input: ParseStream) -> Result<TokenStream> {
    let s = input.parse::<LitStr>()?.value();
    let chars = s.chars();
    Ok(quote! {
        (#(irisia::PChar::<#chars>,)*)
    })
}
