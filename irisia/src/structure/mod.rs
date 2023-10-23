use crate::{update_with::UpdateWith, Result};

pub use self::{
    chain::Chain,
    once::Once,
    repeat::Repeat,
    select::{SelectBody, SelectHead},
};

pub mod chain;
pub mod empty;
pub mod once;
pub mod repeat;
pub mod select;
pub(crate) mod slot;

pub trait VisitLen: Sized {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait VisitBy<V>: VisitLen {
    fn visit(&self, visitor: &mut V) -> Result<()>;
}

pub trait Visitor<T>: Sized {
    fn visit(&mut self, data: &T) -> Result<()>;
}

pub trait VisitMutBy<V>: VisitLen {
    fn visit_mut(&mut self, visitor: &mut V) -> Result<()>;
}

pub trait VisitorMut<T>: Sized {
    fn visit_mut(&mut self, data: &mut T) -> Result<()>;
}
