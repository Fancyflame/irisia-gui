use tokio::sync::RwLockMappedWriteGuard;

use std::ops::{Deref, DerefMut};

use crate::{element::RcElementModel, style::StyleContainer, Element};

pub struct ElWriteGuard<'a, El, Sd: SetDirty> {
    pub(super) write: RwLockMappedWriteGuard<'a, El>,
    pub(super) set_dirty: &'a Sd,
}

impl<El, Sd: SetDirty> Deref for ElWriteGuard<'_, El, Sd> {
    type Target = El;
    fn deref(&self) -> &Self::Target {
        &self.write
    }
}

impl<El, Sd: SetDirty> DerefMut for ElWriteGuard<'_, El, Sd> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.write
    }
}

impl<El, Sd: SetDirty> Drop for ElWriteGuard<'_, El, Sd> {
    fn drop(&mut self) {
        self.set_dirty._set_dirty();
    }
}

pub trait SetDirty {
    fn _set_dirty(&self);
}

impl<El, Sty> SetDirty for RcElementModel<El, Sty>
where
    El: Element,
    Sty: StyleContainer + 'static,
{
    fn _set_dirty(&self) {
        self.set_dirty();
    }
}
