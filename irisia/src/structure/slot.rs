use std::cell::{RefCell, RefMut};
use std::rc::Rc;

use crate::dom::ChildNodes;
use crate::update_with::SpecificUpdate;

use super::UpdateWith;

pub struct Slot<T>(pub Rc<RefCell<T>>);

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
    pub fn new(value: T) -> Self {
        Slot(Rc::new(RefCell::new(value)))
    }

    pub fn borrow_mut(&self) -> RefMut<T> {
        self.0.borrow_mut()
    }
}

impl<T> ChildNodes for Slot<T>
where
    T: ChildNodes,
{
    fn render(
        &self,
        lr: &mut crate::dom::layer::LayerRebuilder,
        interval: std::time::Duration,
    ) -> crate::Result<()> {
        self.0.borrow().render(lr, interval)
    }

    fn emit_event(&self, npe: &crate::application::event_comp::NewPointerEvent) -> bool {
        self.0.borrow().emit_event(npe)
    }

    fn layout(
        &self,
        f: &mut dyn FnMut(
            &dyn crate::style::style_box::InsideStyleBox,
        ) -> Option<crate::primitive::Region>,
    ) -> crate::Result<()> {
        self.0.borrow().layout(f)
    }

    fn len(&self) -> usize {
        self.0.borrow().len()
    }

    fn peek_styles(&self, f: &mut dyn FnMut(&dyn crate::style::style_box::InsideStyleBox)) {
        self.0.borrow().peek_styles(f)
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl<T> SpecificUpdate for &Slot<T> {
    type UpdateTo = Slot<T>;
}
