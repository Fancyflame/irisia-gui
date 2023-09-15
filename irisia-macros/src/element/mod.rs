use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse_quote;

use crate::expr::{conditional::ca::DefaultConditionalApplicator, Codegen};

pub mod build;
pub mod stmt;

pub struct ElementCodegen;

impl Codegen for ElementCodegen {
    type Ca = DefaultConditionalApplicator;
    type Command = Cmd;
    type Stmt = stmt::ElementStmt;

    const MUST_IN_BLOCK: bool = false;

    fn empty() -> TokenStream {
        quote!(())
    }

    fn repetitive_applicate(iter: impl ToTokens) -> TokenStream {
        quote! {
            irisia::structure::Repeat::new(#iter)
        }
    }

    fn conditional_applicate(count: usize) -> Self::Ca {
        DefaultConditionalApplicator::new(count, parse_quote!(irisia::structure::Branch))
    }

    fn parse_command(_: &str, _: syn::parse::ParseStream) -> syn::Result<Option<Self::Command>> {
        Ok(None)
    }

    fn chain_applicate(tokens: &mut TokenStream, other: impl ToTokens) {
        *tokens = quote!(irisia::structure::Chain::new(#tokens), #other);
    }
}

pub struct Cmd;

impl ToTokens for Cmd {
    fn to_tokens(&self, _: &mut proc_macro2::TokenStream) {
        unreachable!();
    }
}
