use std::{
    cell::RefMut,
    ops::{Deref, DerefMut},
};

use crate::{
    element::RcElementModel,
    structure::{SlotUpdater, VisitBy},
    Element,
};

pub struct ElWriteGuard<'a, El, Sty, Slt>
where
    El: Element,
    Slt: VisitBy + 'static,
{
    pub(super) el: RefMut<'a, El>,
    pub(super) model: &'a RcElementModel<El, Sty, Slt>,
}

impl<El, Sty, Slt> Deref for ElWriteGuard<'_, El, Sty, Slt>
where
    El: Element,
    Slt: VisitBy + 'static,
{
    type Target = El;
    fn deref(&self) -> &Self::Target {
        &self.el
    }
}

impl<El, Sty, Slt> DerefMut for ElWriteGuard<'_, El, Sty, Slt>
where
    El: Element,
    Slt: VisitBy + 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.el
    }
}

impl<El, Sty, Slt> Drop for ElWriteGuard<'_, El, Sty, Slt>
where
    El: Element,
    Slt: VisitBy + 'static,
{
    fn drop(&mut self) {
        let updater = self.el.children(SlotUpdater(&self.model.slot));
        self.model.in_cell.borrow_mut().children.update(updater);
    }
}
