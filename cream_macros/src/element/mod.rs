use quote::{quote, ToTokens};
use syn::{parse_quote, token::Paren};

use crate::expr::{conditional::ca::DefaultConditionalApplicator, Codegen};

use self::cmd::ElementCommand;

pub mod build;
pub mod cmd;
pub mod stmt;

pub struct ElementCodegen;

impl Codegen for ElementCodegen {
    type Ca = DefaultConditionalApplicator;
    type Command = cmd::ElementCommand;
    type Stmt = stmt::ElementStmt;

    const IN_BLOCK: bool = false;

    fn empty(tokens: &mut proc_macro2::TokenStream) {
        quote!(cream::structure::EmptyStructure).to_tokens(tokens);
    }

    fn repetitive_applicate<F>(tokens: &mut proc_macro2::TokenStream, f: F)
    where
        F: FnOnce(&mut proc_macro2::TokenStream),
    {
        quote! {
            cream::structure::Repeating::new
        }
        .to_tokens(tokens);
        Paren::default().surround(tokens, f);
    }

    fn conditional_applicate(count: usize) -> Self::Ca {
        DefaultConditionalApplicator::new(count, parse_quote!(cream::structure::Branch))
    }

    fn parse_command(
        cmd: &str,
        input: syn::parse::ParseStream,
    ) -> syn::Result<Option<Self::Command>> {
        Ok(Some(match cmd {
            "slot" => ElementCommand::Slot(input.parse()?),
            "init" => ElementCommand::Init(input.parse()?),
            _ => return Ok(None),
        }))
    }

    fn chain_applicate<F>(tokens: &mut proc_macro2::TokenStream, f: F)
    where
        F: FnOnce(&mut proc_macro2::TokenStream),
    {
        quote!(.chain).to_tokens(tokens);
        Paren::default().surround(tokens, f);
    }
}
