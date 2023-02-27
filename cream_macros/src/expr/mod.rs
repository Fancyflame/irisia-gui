use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    token::Brace,
    Result, Token,
};

use self::{
    conditional::{state_if::StateIf, state_match::StateMatch},
    repetitive::{state_for::StateForLoop, state_while::StateWhile},
    state_block::StateBlock,
    state_command::StateCommand,
};

pub mod conditional;
pub mod repetitive;
pub mod state_block;
pub mod state_command;

pub trait ConditionalApplicator: Sized {
    fn apply<F>(&mut self, tokens: &mut TokenStream, f: F)
    where
        F: FnOnce(&mut TokenStream);
}

pub trait Codegen {
    type Stmt: Parse + ToTokens;
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

pub enum StateExpr<T>
where
    T: Codegen,
{
    Raw(T::Stmt),
    Block(StateBlock<T>),
    If(StateIf<T>),
    Match(StateMatch<T>),
    While(StateWhile<T>),
    For(StateForLoop<T>),
    Command(StateCommand<T>),
}

impl<T: Codegen> Parse for StateExpr<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let r = if input.peek(Token![if]) {
            StateExpr::If(input.parse()?)
        } else if input.peek(Token![match]) {
            StateExpr::Match(input.parse()?)
        } else if input.peek(Token![while]) {
            StateExpr::While(input.parse()?)
        } else if input.peek(Token![for]) {
            StateExpr::For(input.parse()?)
        } else if input.peek(Brace) {
            StateExpr::Block(input.parse()?)
        } else if input.peek(Token!(@)) {
            StateExpr::Command(input.parse()?)
        } else {
            StateExpr::Raw(input.parse()?)
        };
        Ok(r)
    }
}

impl<T: Codegen> ToTokens for StateExpr<T> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        macro_rules! to_tokens {
            ($($Arm:ident)*) => {
                match self {
                    $(StateExpr::$Arm(x) => x.to_tokens(tokens),)*
                }
            };
        }

        to_tokens!(Raw Block If Match While For Command);
    }
}
