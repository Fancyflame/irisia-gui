use quote::ToTokens;

pub use self::{state_for::StateForLoop, state_while::StateWhile};
use super::{CodegenAlias, StateBlock, StmtTree};

pub mod state_for;
pub mod state_while;

pub enum StateRepetitive<T: StmtTree> {
    For(StateForLoop<T>),
    While(StateWhile<T>),
}

impl<T: StmtTree> StateRepetitive<T> {
    pub fn body(&self) -> &StateBlock<T> {
        match self {
            Self::For(f) => &f.body,
            Self::While(w) => &w.state_block,
        }
    }
}

impl<T: CodegenAlias> ToTokens for StateRepetitive<T> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let expr_iter = match self {
            Self::For(f) => f.expr_iter(),
            Self::While(w) => w.expr_iter(),
        };

        tokens.extend(T::repetitive_applicate(expr_iter))
    }
}
