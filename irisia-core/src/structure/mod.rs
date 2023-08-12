use crate::{update_with::UpdateWith, Result};

pub mod branch;
pub mod chain;
pub mod empty;
pub mod once;
pub mod repeating;
pub(crate) mod slot;

pub trait VisitLen: Sized {
    fn len(&self) -> usize;
}

macro_rules! visit_trait {
    ($Visit:ident $Visitor:ident $visit:ident $($mut:ident)?) => {
        pub trait $Visit<V>: VisitLen {
            fn $visit(& $($mut)? self, visitor: &mut V) -> Result<()>;
        }

        pub trait $Visitor<T>: Sized {
            fn $visit(&mut self, data: & $($mut)? T) -> Result<()>;
        }
    };
}

pub trait MapVisit<V> {
    type Output;
    fn map(self, visitor: &V) -> Self::Output;
}

pub trait MapVisitor<T> {
    type Output;
    fn map_visit(&self, data: T) -> Self::Output;
}

visit_trait!(Visit Visitor visit);
visit_trait!(VisitMut VisitorMut visit_mut mut);
