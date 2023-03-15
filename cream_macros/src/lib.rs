use expr::{state_block::parse_stmts, state_block::StateBlock};
use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use style::StyleCodegen;
use syn::{
    parse::{ParseStream, Parser},
    parse_macro_input, DeriveInput,
};

mod derive_style;
mod element;
pub(crate) mod expr;
mod style;

#[proc_macro]
pub fn build(input: TokenStream) -> TokenStream {
    let helper = |input: ParseStream| element::build::build(input, false);
    match helper.parse(input) {
        Ok(t) => t.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn render(input: TokenStream) -> TokenStream {
    let helper = |input: ParseStream| element::build::build(input, true);
    match helper.parse(input) {
        Ok(t) => t.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn style(input: TokenStream) -> TokenStream {
    let stmts = match parse_stmts::<StyleCodegen>.parse(input) {
        Ok(stmts) => stmts,
        Err(e) => return e.to_compile_error().into(),
    };

    let stream = StateBlock {
        brace: Default::default(),
        stmts,
    }
    .into_token_stream();
    stream.into()
}

#[proc_macro_derive(Event)]
pub fn event_derive(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident, generics, ..
    } = parse_macro_input!(input as DeriveInput);

    let (impl_gen, type_gen, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_gen ::cream_core::Event for #ident #type_gen
        #where_clause
        {}
    }
    .into()
}

#[proc_macro_derive(Style, attributes(cream))]
pub fn derive_style(input: TokenStream) -> TokenStream {
    match derive_style::derive_style(parse_macro_input!(input as DeriveInput)) {
        Ok(t) => t.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
