use std::marker::PhantomData;

use crate::update_with::SpecificUpdate;

use super::MapVisit;
use super::{UpdateWith, VisitLen};

pub struct SlotModel<T>(pub(crate) T);

pub(crate) struct Slot<T>(pub T);
pub(crate) struct SlotNotUpdate<T>(pub PhantomData<T>);

impl<T, V> MapVisit<V> for Slot<T>
where
    T: MapVisit<V>,
{
    type Output = Slot<T::Output>;
    fn map(self, visitor: &V) -> Self::Output {
        Slot(self.0.map(visitor))
    }
}

impl<T, V> MapVisit<V> for SlotNotUpdate<T> {
    type Output = Self;
    fn map(self, _: &V) -> Self::Output {
        self
    }
}

impl<T> VisitLen for SlotModel<T>
where
    T: VisitLen,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T, U> UpdateWith<Slot<U>> for SlotModel<T>
where
    T: UpdateWith<U>,
{
    fn create_with(updater: Slot<U>) -> Self {
        SlotModel(T::create_with(updater.0))
    }

    fn update_with(&mut self, updater: Slot<U>, equality_matters: bool) -> bool {
        self.0.update_with(updater.0, equality_matters)
    }
}

impl<T> UpdateWith<SlotNotUpdate<T>> for SlotModel<T> {
    fn create_with(_: SlotNotUpdate<T>) -> Self {
        panic!("inner error: cannot create slot model with empty slot");
    }

    fn update_with(&mut self, _: SlotNotUpdate<T>, equality_matters: bool) -> bool {
        equality_matters
    }
}

impl<T> SpecificUpdate for Slot<T>
where
    T: SpecificUpdate,
{
    type UpdateTo = SlotModel<T::UpdateTo>;
}

impl<T> SpecificUpdate for SlotNotUpdate<T> {
    type UpdateTo = SlotModel<T>;
}
