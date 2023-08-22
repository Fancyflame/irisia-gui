use std::cell::RefCell;
use std::rc::Rc;

use crate::dom::EMUpdateContent;
use crate::update_with::SpecificUpdate;

use super::MapVisit;
use super::{UpdateWith, VisitLen};

pub(crate) struct Slot<T>(pub Rc<RefCell<T>>);

impl<T> MapVisit<EMUpdateContent<'_>> for &Slot<T> {
    type Output = Self;
    fn map(self, _: &EMUpdateContent) -> Self {
        self
    }
}

impl<T> VisitLen for Slot<T>
where
    T: VisitLen,
{
    fn len(&self) -> usize {
        self.0.borrow().len()
    }
}

impl<T> UpdateWith<&Slot<T>> for Slot<T> {
    fn create_with(updater: &Slot<T>) -> Self {
        Slot(updater.0.clone())
    }

    fn update_with(&mut self, updater: &Slot<T>, equality_matters: bool) -> bool {
        if Rc::ptr_eq(&self.0, &updater.0) {
            self.0 = updater.0.clone();
            false
        } else {
            equality_matters
        }
    }
}

impl<T> Slot<T> {
    pub fn new<U>(value: U) -> Self
    where
        T: UpdateWith<U>,
    {
        Slot(Rc::new(RefCell::new(T::create_with(value))))
    }

    pub fn update_inner<U>(&self, value: U, equality_matters: bool) -> bool
    where
        T: UpdateWith<U>,
    {
        self.0.borrow_mut().update_with(value, equality_matters)
    }
}

impl<T> SpecificUpdate for &Slot<T> {
    type UpdateTo = Slot<T>;
}
