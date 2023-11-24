use crate::{dom::DropProtection, style::StyleGroup, update_with::UpdateWith, Element, Result};

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

pub trait VisitBy {
    fn visit_by<V>(&self, visitor: &mut V) -> Result<()>
    where
        V: VisitOn;

    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait VisitOn {
    fn visit_on<El>(&mut self, data: &DropProtection<El>) -> Result<()>
    where
        El: Element;
}
