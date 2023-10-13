use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::ParseStream, Result};

use crate::expr::{enum_conditional, state_block::parse_stmts, Codegen, StateExpr};

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

    const MUST_IN_BLOCK: bool = true;

    fn parse_command(_cmd: &str, _input: ParseStream) -> syn::Result<Option<Self::Command>> {
        Ok(None)
    }

    fn empty() -> TokenStream {
        quote!(())
    }

    fn repetitive_applicate(_: impl ToTokens) -> TokenStream {
        quote!(::std::compile_error!(
            "repetitive structure is not allowed in style macro"
        ))
    }

    fn conditional_applicate(stmt: impl ToTokens, index: usize, total: usize) -> TokenStream {
        enum_conditional(
            quote!(irisia::style::Branch::ArmA),
            quote!(irisia::style::Branch::ArmB),
            stmt,
            index,
            total,
        )
    }

    fn chain_applicate(prev: impl ToTokens, after: impl ToTokens) -> TokenStream {
        quote!(irisia::style::Chain::new(#prev, #after))
    }
}

pub fn style(input: ParseStream) -> Result<Vec<StateExpr<StyleCodegen>>> {
    let mut stmts = parse_stmts(input)?;
    handle_style_follow(&mut stmts)?;
    Ok(stmts)
}
