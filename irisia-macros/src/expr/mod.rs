use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    token::Brace,
    Result, Token,
};

pub use self::{
    conditional::{state_if::StateIf, state_match::StateMatch},
    repetitive::StateRepetitive,
    state_block::StateBlock,
    state_command::{StateCommand, StateCommandBody},
    state_expr::StateExpr,
};

pub mod conditional;
pub mod repetitive;
pub mod state_block;
pub mod state_command;
pub mod state_expr;

pub trait StmtTree {
    type Stmt: Parse;
    type Command;

    const MUST_IN_BLOCK: bool;

    fn parse_command(_cmd: &str, _input: ParseStream) -> Result<Option<Self::Command>> {
        Ok(None)
    }
}

pub trait StmtTreeCodegen
where
    Self: StmtTree,
    Self::Stmt: ToTokens,
{
    fn empty() -> TokenStream;

    fn command_applicate(_cmd: &Self::Command) -> Option<TokenStream> {
        None
    }

    fn conditional_applicate(stmt: impl ToTokens, index: usize, total: usize) -> TokenStream;

    fn repetitive_applicate(stmt: impl ToTokens) -> TokenStream;

    fn chain_applicate(prev: impl ToTokens, after: impl ToTokens) -> TokenStream;
}

pub trait CodegenAlias: StmtTreeCodegen + StmtTree<Stmt = Self::StmtAlias> {
    type StmtAlias: ToTokens;
}

impl<T> CodegenAlias for T
where
    T: StmtTreeCodegen + StmtTree,
    T::Stmt: Parse + ToTokens,
{
    type StmtAlias = T::Stmt;
}

pub fn enum_conditional(
    branch1: impl ToTokens,
    branch2: impl ToTokens,
    stmt: impl ToTokens,
    index: usize,
    total: usize,
) -> TokenStream {
    // if it is the last branch
    let mut output = if index == total - 1 {
        stmt.to_token_stream()
    } else {
        quote!(#branch1(#stmt))
    };

    for _ in 0..index {
        output = quote!(#branch2(#output));
    }

    output
}
