use std::{cell::RefCell, rc::Rc};

use crate::dep_watch::bitset::UsizeArray;

use super::{StructureUpdateTo, VisitBy, VisitOn};

pub struct Slot<T>(Rc<RefCell<T>>);
pub struct SlotUpdater<'a, T>(pub(crate) &'a Slot<T>);

impl<T> Slot<T> {
    pub(crate) fn new(v: T) -> Self {
        Self(Rc::new(RefCell::new(v)))
    }

    pub(crate) fn update<F>(&self, update: F)
    where
        F: FnOnce(&mut T),
    {
        update(&mut self.0.borrow_mut())
    }

    pub(crate) fn private_clone(&self) -> Self {
        Slot(self.0.clone())
    }
}

impl<T> VisitBy for Slot<T>
where
    T: VisitBy,
{
    fn visit_by<V>(&self, visitor: &mut V) -> crate::Result<()>
    where
        V: VisitOn,
    {
        self.0.borrow().visit_by(visitor)
    }

    fn len(&self) -> usize {
        self.0.borrow().len()
    }

    fn is_empty(&self) -> bool {
        self.0.borrow().is_empty()
    }
}

impl<T, A: UsizeArray> StructureUpdateTo<A> for SlotUpdater<'_, T>
where
    T: VisitBy + 'static,
{
    type Target = Slot<T>;
    const UPDATE_POINTS: u32 = 0;

    fn create(self, _: super::Updating<A>) -> Self::Target {
        self.0.private_clone()
    }

    fn update(self, target: &mut Self::Target, _: super::Updating<A>) {
        if !Rc::ptr_eq(&self.0 .0, &target.0) {
            *target = self.0.private_clone();
        }
    }
}
