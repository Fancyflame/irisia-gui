use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::ParseStream, parse_quote, token::Paren, Result};

use crate::expr::{
    conditional::ca::DefaultConditionalApplicator, state_block::parse_stmts, Codegen, StateExpr,
};

use self::stmt::{handle_style_follow, StyleStmt};

pub mod stmt;

pub struct StyleCommand(());

impl ToTokens for StyleCommand {
    fn to_tokens(&self, _tokens: &mut TokenStream) {
        unreachable!();
    }
}

pub struct StyleCodegen;

impl Codegen for StyleCodegen {
    type Command = StyleCommand;
    type Stmt = StyleStmt;
    type Ca = DefaultConditionalApplicator;

    const MUST_IN_BLOCK: bool = true;

    fn parse_command(_cmd: &str, _input: ParseStream) -> syn::Result<Option<Self::Command>> {
        Ok(None)
    }

    fn empty(tokens: &mut TokenStream) {
        quote!(irisia::style::NoStyle).to_tokens(tokens);
    }

    fn repetitive_applicate<F>(tokens: &mut TokenStream, _: F)
    where
        F: FnOnce(&mut TokenStream),
    {
        quote!(::std::compile_error!(
            "repetitive structure is not allowed in style macro"
        ))
        .to_tokens(tokens);
    }

    fn conditional_applicate(count: usize) -> Self::Ca {
        DefaultConditionalApplicator::new(count, parse_quote!(irisia::style::Branch))
    }

    fn chain_applicate<F>(tokens: &mut TokenStream, f: F)
    where
        F: FnOnce(&mut TokenStream),
    {
        let mut stream = quote!(.chain);
        Paren::default().surround(&mut stream, f);
        stream.to_tokens(tokens);
    }
}

pub fn style(input: ParseStream) -> Result<Vec<StateExpr<StyleCodegen>>> {
    let mut stmts = parse_stmts(input)?;
    handle_style_follow(&mut stmts)?;
    Ok(stmts)
}
