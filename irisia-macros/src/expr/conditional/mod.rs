use quote::ToTokens;

use super::{Codegen, StateExpr, StateIf, StateMatch};

pub mod state_if;
pub mod state_match;

pub enum StateConditional<T: Codegen> {
    If(StateIf<T>),
    Match(StateMatch<T>),
}

impl<T: Codegen> StateConditional<T> {
    pub fn arms(&self) -> impl Iterator<Item = &[StateExpr<T>]> {
        enum Branch<T, U> {
            T(T),
            U(U),
        }

        let mut iter = match self {
            Self::If(i) => Branch::T(i.arms()),
            Self::Match(m) => Branch::U(m.arms()),
        };

        std::iter::from_fn(move || match &mut iter {
            Branch::T(i) => i.next().map(|block| &*block.stmts),
            Branch::U(m) => m.next().map(|expr| std::array::from_ref(expr) as _),
        })
    }
}

impl<T: Codegen> ToTokens for StateConditional<T> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::If(i) => i.to_tokens(tokens),
            Self::Match(m) => m.to_tokens(tokens),
        }
    }
}
