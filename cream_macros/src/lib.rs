use expr::state_block::{parse_stmts, stmts_to_tokens};
use match_data::ExprMatchData;
use proc_macro::TokenStream;
use quote::quote;
use style::StyleCodegen;
use syn::{
    parse::{ParseStream, Parser},
    parse_macro_input, DeriveInput,
};

mod derive_style;
mod element;
pub(crate) mod expr;
mod match_data;
mod style;

#[proc_macro]
pub fn build(input: TokenStream) -> TokenStream {
    let helper = |input: ParseStream| element::build::build(input);
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

    let mut stmt_tokens = proc_macro2::TokenStream::new();
    stmts_to_tokens(&mut stmt_tokens, &stmts);

    quote! {{
        use cream_core::style::StyleContainer;
        #stmt_tokens
    }}
    .into()
}

#[proc_macro_derive(Event)]
pub fn derive_event(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident, generics, ..
    } = parse_macro_input!(input as DeriveInput);

    let (impl_gen, type_gen, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_gen cream_core::Event for #ident #type_gen
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

#[proc_macro]
pub fn match_event(input: TokenStream) -> TokenStream {
    let match_data = parse_macro_input!(input as ExprMatchData);
    match_data::expand(match_data).into()
}
