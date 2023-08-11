use irisia_backend::WinitWindow;
use std::{
    ops::{Deref, DerefMut},
    sync::{atomic::AtomicBool, Arc},
};
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{application::content::GlobalContent, event::EventDispatcher, UpdateWith};

pub struct ElementHandle<El> {
    dirty_setter: SetDirty,
    el: RwLock<El>,
    ed: EventDispatcher,
    global_content: Arc<GlobalContent>,
}

impl<El> ElementHandle<El> {
    pub(super) fn new<U>(gc: Arc<GlobalContent>, updater: U) -> Self
    where
        El: UpdateWith<U>,
    {
        Self {
            dirty_setter: SetDirty(Arc::new(AtomicBool::new(false))),
            el: RwLock::new(El::create_with(updater)),
            ed: EventDispatcher::new(),
            global_content: gc,
        }
    }

    pub(super) fn el_mut(&self) -> RwLockWriteGuard<El> {
        self.el.blocking_write()
    }

    pub async fn el_write(&self) -> ElWriteGuard<El> {
        ElWriteGuard {
            write: self.el.write().await,
            set_dirty: &self.dirty_setter,
        }
    }

    pub async fn el_read(&self) -> RwLockReadGuard<El> {
        self.el.read().await
    }

    pub fn event_dispatcher(&self) -> &EventDispatcher {
        &self.ed
    }

    pub fn focus(&self) {
        self.global_content.focusing().focus(self.ed.clone());
    }

    pub fn blur(&self) {
        self.global_content.focusing().blur_checked(&self.ed);
    }

    pub fn global(&self) -> &Arc<GlobalContent> {
        &self.global_content
    }

    pub fn window(&self) -> &WinitWindow {
        self.global_content.window()
    }

    pub fn set_dirty(&self) {
        self.dirty_setter.set();
    }

    pub fn dirty_setter(&self) -> SetDirty {
        self.dirty_setter.clone()
    }
}

#[derive(Clone)]
pub struct SetDirty(pub(crate) Arc<AtomicBool>);

impl SetDirty {
    pub fn set(&self) {
        self.0.store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

pub struct ElWriteGuard<'a, T> {
    write: RwLockWriteGuard<'a, T>,
    set_dirty: &'a SetDirty,
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
        self.set_dirty.set()
    }
}
