use std::{cell::RefCell, rc::Rc};

use super::VisitBy;

pub struct Slot<T>(Rc<RefCell<T>>);

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
        V: super::VisitOn,
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
