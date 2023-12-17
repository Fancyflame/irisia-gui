use crate::{update_with::UpdateWith, ElModel, Result};

pub use self::{
    chain::Chain,
    once::Once,
    repeat::Repeat,
    select::{SelectBody, SelectHead},
    slot::Slot,
};

mod chain;
mod empty;
mod once;
mod repeat;
pub mod select;
mod slot;

pub trait VisitBy {
    fn visit_by<V>(&self, visitor: &mut V) -> Result<()>
    where
        V: VisitOn;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

type UpdateSlotFn<'a, 'b, Slt> = &'a mut dyn FnMut(UpdateNode<'b, Slt>);

pub trait UpdateSlot<Slt> {
    fn will_update() -> bool
    where
        Self: Sized;
    fn update_slot(&mut self, f: UpdateSlotFn<Slt>);
}

pub enum UpdateNode<'a, T> {
    NeedsInit(&'a mut Option<T>),
    NeedsUpdate(&'a mut T),
}

pub trait VisitOn {
    fn visit_on(&mut self, data: &ElModel!(_)) -> Result<()>;
}
