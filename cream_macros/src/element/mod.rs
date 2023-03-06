use quote::{quote, ToTokens};
use syn::{parenthesized, parse::ParseStream, parse_quote, token::Paren, Result, Token};

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
        quote!(::cream_core::structure::EmptyStructure).to_tokens(tokens);
    }

    fn repetitive_applicate<F>(tokens: &mut proc_macro2::TokenStream, f: F)
    where
        F: FnOnce(&mut proc_macro2::TokenStream),
    {
        quote! {
            ::cream_core::structure::Repeating::new
        }
        .to_tokens(tokens);
        Paren::default().surround(tokens, f);
    }

    fn conditional_applicate(count: usize) -> Self::Ca {
        DefaultConditionalApplicator::new(count, parse_quote!(::cream_core::structure::Branch))
    }

    fn parse_command(
        cmd: &str,
        input: syn::parse::ParseStream,
    ) -> syn::Result<Option<Self::Command>> {
        Ok(Some(match cmd {
            "slot" => ElementCommand::Slot(input.parse()?),
            "init" => parse_init_command(input)?,
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

fn parse_init_command(input: ParseStream) -> Result<ElementCommand> {
    let content;
    parenthesized!(content in input);
    let event_src = content.parse()?;

    Ok(if content.peek(Token![,]) {
        ElementCommand::InitRender {
            event_src,
            cache_box: {
                content.parse::<Token![,]>()?;
                content.parse()?
            },
            render_content: {
                content.parse::<Token![,]>()?;
                content.parse()?
            },
        }
    } else {
        ElementCommand::InitBuild { event_src }
    })
}
