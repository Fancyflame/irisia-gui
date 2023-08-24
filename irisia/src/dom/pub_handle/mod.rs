use irisia_backend::WinitWindow;
use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};
use tokio::sync::{RwLockMappedWriteGuard, RwLockReadGuard, RwLockWriteGuard};

use crate::{application::content::GlobalContent, event::EventDispatcher};

use self::listen::Listen;

use super::data_structure::ElementHandle;

pub mod listen;

impl<El> ElementHandle<El> {
    /// Get a write guard without setting dirty
    pub(super) fn el_write_clean(&self) -> RwLockMappedWriteGuard<El> {
        RwLockWriteGuard::map(self.el.blocking_write(), |x| x.as_mut().unwrap())
    }

    /// Get a write guard of this element and setting dirty.
    pub async fn el_write(&self) -> ElWriteGuard<El> {
        ElWriteGuard {
            write: RwLockWriteGuard::map(self.el.write().await, |x| x.as_mut().unwrap()),
            set_dirty: self,
        }
    }

    /// Get a read guard of this element and dirty flag is not affected.
    pub async fn el_read(&self) -> RwLockReadGuard<El> {
        RwLockReadGuard::map(self.el.read().await, |x| x.as_ref().unwrap())
    }

    /// Get event dispatcher of this element.
    pub fn event_dispatcher(&self) -> &EventDispatcher {
        &self.ed
    }

    /// Let this element being focused on.
    pub fn focus(&self) {
        self.global_content.focusing().focus(self.ed.clone());
    }

    /// Let this element no longer being focused. does nothing if
    /// this element is not in focus.
    pub fn blur(&self) {
        self.global_content.focusing().blur_checked(&self.ed);
    }

    /// Get global content of the window.
    pub fn global(&self) -> &Arc<GlobalContent> {
        &self.global_content
    }

    /// Get the raw window. Alias to `self.global().window()`.
    pub fn window(&self) -> &WinitWindow {
        self.global_content.window()
    }

    /// Set dirty flag to `true`.
    pub fn set_dirty(&self) {
        self.global_content
            .request_redraw(self.layer_info.read().unwrap().render_layer_id())
    }

    /// Listen event with options
    pub fn listen<'a>(self: &'a Arc<Self>) -> Listen<'a, El, (), (), (), (), ()> {
        Listen::new(self)
    }

    /// Set `true` to acquire independent render layer for performance optimizations.
    /// Duplicated acquirement will be ignored.
    ///
    /// A independent render layer will cause a heap allocation to render this
    /// element and its children elements rather than drawing on parent's render
    /// layer.
    ///
    /// Recommended when playing animation.
    pub fn acquire_independent_layer(&self, acquire: bool) {
        let mut write = self.layer_info.write().unwrap();
        if write.acquire_independent_layer == acquire {
            return;
        }

        write.acquire_independent_layer = acquire;
        self.global_content.request_redraw(write.parent_layer_id);
    }

    /// Query whether independent layer was acquired.
    ///
    /// Note that this function will NOT reflect the actual state of
    /// the layer. Independent layer may not exists while result is `true`
    /// and may exists while result is `false`.
    pub fn independent_layer_acquired(&self) -> bool {
        self.layer_info.read().unwrap().acquire_independent_layer
    }
}

pub struct ElWriteGuard<'a, T> {
    write: RwLockMappedWriteGuard<'a, T>,
    set_dirty: &'a ElementHandle<T>,
}

impl<T> Deref for ElWriteGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.write
    }
}

impl<T> DerefMut for ElWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.write
    }
}

impl<T> Drop for ElWriteGuard<'_, T> {
    fn drop(&mut self) {
        self.set_dirty.set_dirty();
    }
}
