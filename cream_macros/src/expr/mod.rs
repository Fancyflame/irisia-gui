use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    token::Brace,
    Result, Token,
};

pub use self::{
    conditional::{state_if::StateIf, state_match::StateMatch},
    repetitive::{state_for::StateForLoop, state_while::StateWhile},
    state_block::StateBlock,
    state_command::{StateCommand, StateCommandBody},
    state_expr::StateExpr,
};

pub mod conditional;
pub mod repetitive;
pub mod state_block;
pub mod state_command;
pub mod state_expr;

pub trait ConditionalApplicator: Sized {
    fn apply<F>(&mut self, tokens: &mut TokenStream, f: F)
    where
        F: FnOnce(&mut TokenStream);
}

pub trait Codegen {
    type Stmt: ToTokens + Parse;
    type Ca: ConditionalApplicator;
    type Command: ToTokens;

    const IN_BLOCK: bool;

    fn parse_command(cmd: &str, input: ParseStream) -> Result<Option<Self::Command>>;

    fn empty(tokens: &mut TokenStream);

    fn conditional_applicate(count: usize) -> Self::Ca;

    fn repetitive_applicate<F>(tokens: &mut TokenStream, f: F)
    where
        F: FnOnce(&mut TokenStream);

    fn chain_applicate<F>(tokens: &mut TokenStream, f: F)
    where
        F: FnOnce(&mut TokenStream);
}

// only visit raw or command
pub trait VisitUnit<T> {
    fn visit_unit<F>(&self, f: &mut F)
    where
        F: FnMut(&StateExpr<T>);

    fn visit_unit_mut<F>(&mut self, f: &mut F)
    where
        F: FnMut(&mut StateExpr<T>);
}

pub trait StateToTokens {
    fn state_to_tokens<C: Codegen>(tokens: TokenStream);
}
