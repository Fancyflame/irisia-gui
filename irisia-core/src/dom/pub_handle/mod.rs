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
            .request_redraw(*self.dep_layer_id.lock().unwrap())
    }

    /// Listen event with options
    pub fn listen<'a>(self: &'a Arc<Self>) -> Listen<'a, El, (), (), (), (), ()> {
        Listen::new(self)
    }

    /*pub fn require_independent_layer(&self, required: bool){
        let old=self.lock_independent_layer.swap(required, std::sync::atomic::Ordering::Relaxed);
        if required != old{
            todo!()
        }
    }*/
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
