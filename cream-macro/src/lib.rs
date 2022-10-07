use parse::Component;
use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2, TokenTree};
use quote::format_ident;

#[macro_use]
extern crate syn;

macro_rules! parse_err{
    ($span:expr,$msg:literal$($tt:tt)*)=>{
        return Err(syn::Error::new($span,format!($msg$($tt)*)))
    };
    ($span:expr,$msg:expr)=>{
        return Err(syn::Error::new($span,$msg))
    };
}

pub(crate) const PRIVATE_PREFIX: &str = "__ChampagneGUIPrivate__";

mod generate;
mod parse;

#[proc_macro]
pub fn champagne(input: TokenStream) -> TokenStream {
    let comp: Component = syn::parse(input).expect("Cannot parse the input");
    generate::assemble::generate(comp)
}
