use crate::{update_with::UpdateWith, Result};

pub use self::{branch::Branch, chain::Chain, once::Once, repeat::Repeat};

pub mod branch;
pub mod chain;
pub mod empty;
pub mod once;
pub mod repeat;
pub(crate) mod slot;

pub trait VisitLen: Sized {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait Visit<V>: VisitLen {
    fn visit(&self, visitor: &mut V) -> Result<()>;
}

pub trait Visitor<T>: Sized {
    fn visit(&mut self, data: &T) -> Result<()>;
}

pub trait VisitMut<V>: VisitLen {
    fn visit_mut(&mut self, visitor: &mut V) -> Result<()>;
}

pub trait VisitorMut<T>: Sized {
    fn visit_mut(&mut self, data: &mut T) -> Result<()>;
}

pub trait MapVisit<V> {
    type Output;
    fn map(self, visitor: &V) -> Self::Output;
}

pub trait MapVisitor<T> {
    type Output;
    fn map_visit(&self, data: T) -> Self::Output;
}
