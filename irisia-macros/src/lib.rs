use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use props2::CastProp;
use quote::quote;
use syn::{DeriveInput, Item, ItemFn, Result, parse::Parser, parse_macro_input};

macro_rules! const_quote {
    (
        $(
            $vis:vis const $Name:ident = {
                $($tt:tt)*
            };
        )*
    ) => {
        $(
            #[doc = "## expand to"]
            #[doc = stringify!($($tt)*)]
            #[allow(non_camel_case_types)]
            $vis struct $Name;

            impl quote::ToTokens for $Name {
                fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
                    quote! { $($tt)* }.to_tokens(tokens)
                }
            }
        )*
    };
}

mod build_macro;
mod generics_unbracketed;
mod inner_impl_listen;
mod main_macro;
mod parse_incomplete;
mod pname;
mod props2;
mod split_generics;
mod style;

#[proc_macro_attribute]
pub fn style(attr: TokenStream, input: TokenStream) -> TokenStream {
    result_into_stream(style::ImplStyle::parse(attr, input).map(|x| x.to_tokens()))
}

#[proc_macro_attribute]
pub fn main(_: TokenStream, input: TokenStream) -> TokenStream {
    let item_fn = parse_macro_input!(input as ItemFn);
    result_into_stream(main_macro::main_macro(item_fn))
}

#[proc_macro_derive(Event)]
pub fn derive_event(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident, generics, ..
    } = parse_macro_input!(input as DeriveInput);

    let (impl_gen, type_gen, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_gen irisia::Event for #ident #type_gen
        #where_clause
        {}
    }
    .into()
}

#[proc_macro_attribute]
pub fn props(_: TokenStream, input: TokenStream) -> TokenStream {
    parse_macro_input!(input as CastProp).generate().into()
}

fn result_into_stream(result: Result<TokenStream2>) -> TokenStream {
    match result {
        Ok(t) => t.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn __inner_impl_listen(_: TokenStream) -> TokenStream {
    inner_impl_listen::impl_listen().into()
}

#[proc_macro]
pub fn build(input: TokenStream) -> TokenStream {
    result_into_stream(build_macro::build_macro.parse(input))
}

#[proc_macro]
pub fn pname(input: TokenStream) -> TokenStream {
    result_into_stream(pname::pname.parse(input))
}
