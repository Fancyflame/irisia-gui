use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::expr::{enum_conditional, Codegen};

pub mod build;
pub mod stmt;

pub struct ElementCodegen;

impl Codegen for ElementCodegen {
    type Command = Cmd;
    type Stmt = stmt::ElementStmt;

    const MUST_IN_BLOCK: bool = false;

    fn empty() -> TokenStream {
        quote!(())
    }

    fn repetitive_applicate(iter: impl ToTokens) -> TokenStream {
        quote! {
            irisia::structure::Repeat(#iter)
        }
    }

    fn conditional_applicate(stmt: impl ToTokens, index: usize, total: usize) -> TokenStream {
        enum_conditional(
            quote!(irisia::structure::Branch::ArmA),
            quote!(irisia::structure::Branch::ArmB),
            stmt,
            index,
            total,
        )
    }

    fn chain_applicate(prev: impl ToTokens, after: impl ToTokens) -> TokenStream {
        quote!(irisia::structure::Chain::new(#prev, #after))
    }
}

pub struct Cmd;

impl ToTokens for Cmd {
    fn to_tokens(&self, _: &mut proc_macro2::TokenStream) {
        unreachable!();
    }
}
