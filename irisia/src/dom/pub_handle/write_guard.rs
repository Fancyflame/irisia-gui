use tokio::sync::RwLockMappedWriteGuard;

use std::ops::{Deref, DerefMut};

use crate::{dom::ChildNodes, element::RcElementModel, style::StyleContainer, Element};

pub struct ElWriteGuard<'a, El, Sd: SetDirty<El>> {
    pub(super) write: RwLockMappedWriteGuard<'a, El>,
    pub(super) set_dirty: &'a Sd,
}

impl<El, Sd: SetDirty<El>> Deref for ElWriteGuard<'_, El, Sd> {
    type Target = El;
    fn deref(&self) -> &Self::Target {
        &self.write
    }
}

impl<El, Sd: SetDirty<El>> DerefMut for ElWriteGuard<'_, El, Sd> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.write
    }
}

impl<El, Sd: SetDirty<El>> Drop for ElWriteGuard<'_, El, Sd> {
    fn drop(&mut self) {
        self.set_dirty._set_dirty(&self.write);
    }
}

pub trait SetDirty<El> {
    fn _set_dirty(&self, el: &El);
}

impl<El, Sty, Sc> SetDirty<El> for RcElementModel<El, Sty, Sc>
where
    El: Element,
    Sty: StyleContainer + 'static,
    Sc: ChildNodes + 'static,
{
    fn _set_dirty(&self, el: &El) {
        el.set_children(self);
        self.set_dirty();
    }
}
