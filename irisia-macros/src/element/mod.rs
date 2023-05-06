use quote::{quote, ToTokens};
use syn::{parse_quote, token::Paren};

use crate::expr::{conditional::ca::DefaultConditionalApplicator, Codegen};

pub mod build;
pub mod stmt;

pub struct ElementCodegen;

impl Codegen for ElementCodegen {
    type Ca = DefaultConditionalApplicator;
    type Command = Cmd;
    type Stmt = stmt::ElementStmt;

    const MUST_IN_BLOCK: bool = false;

    fn empty(tokens: &mut proc_macro2::TokenStream) {
        quote!(irisia::structure::EmptyStructure).to_tokens(tokens);
    }

    fn repetitive_applicate<F>(tokens: &mut proc_macro2::TokenStream, f: F)
    where
        F: FnOnce(&mut proc_macro2::TokenStream),
    {
        quote! {
            irisia::structure::Repeating::new
        }
        .to_tokens(tokens);
        Paren::default().surround(tokens, f);
    }

    fn conditional_applicate(count: usize) -> Self::Ca {
        DefaultConditionalApplicator::new(count, parse_quote!(irisia::structure::Branch))
    }

    fn parse_command(_: &str, _: syn::parse::ParseStream) -> syn::Result<Option<Self::Command>> {
        Ok(None)
    }

    fn chain_applicate<F>(tokens: &mut proc_macro2::TokenStream, f: F)
    where
        F: FnOnce(&mut proc_macro2::TokenStream),
    {
        quote!(.chain).to_tokens(tokens);
        Paren::default().surround(tokens, f);
    }
}

pub struct Cmd;

impl ToTokens for Cmd {
    fn to_tokens(&self, _: &mut proc_macro2::TokenStream) {
        unreachable!();
    }
}
