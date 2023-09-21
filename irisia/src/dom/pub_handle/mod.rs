use irisia_backend::WinitWindow;
use std::{
    cell::RefMut,
    future::Future,
    ops::{Deref, DerefMut},
    rc::Rc,
};
use tokio::{
    sync::{RwLockMappedWriteGuard, RwLockReadGuard, RwLockWriteGuard},
    task::JoinHandle,
};

use crate::{
    application::content::GlobalContent, event::EventDispatcher, primitive::Region,
    structure::slot::Slot, style::StyleContainer, Element, StyleReader,
};

pub use self::{layout_el::LayoutElements, listen::Listen};

use super::{
    children::ChildrenNodes, data_structure::ElementModel, RcElementModel, RenderMultiple,
};

mod layout_el;
mod listen;

impl<El, Sty, Sc> ElementModel<El, Sty, Sc> {
    /// Get a write guard without setting dirty
    pub(super) fn el_write_clean(&self) -> RwLockMappedWriteGuard<El> {
        RwLockWriteGuard::map(self.el.blocking_write(), |x| x.as_mut().unwrap())
    }

    /// Get a write guard of this element and setting dirty.
    pub async fn el_write<'a>(self: &'a Rc<Self>) -> ElWriteGuard<'a, El, Sty, Sc>
    where
        El: Element,
    {
        ElWriteGuard {
            write: RwLockWriteGuard::map(self.el.write().await, |x| x.as_mut().unwrap()),
            set_dirty: self,
        }
    }

    /// Get a read guard of this element and dirty flag is not affected.
    pub async fn el_read(&self) -> RwLockReadGuard<El> {
        RwLockReadGuard::map(self.el.read().await, |x| x.as_ref().unwrap())
    }

    /// Listen event with options
    pub fn listen<'a>(self: &'a Rc<Self>) -> Listen<&'a Rc<Self>, (), (), (), ()> {
        Listen::new(self)
    }

    pub fn slot(&self) -> impl ChildrenNodes + '_
    where
        Slot<Sc>: RenderMultiple,
    {
        &self.slot_cache
    }

    #[must_use]
    pub fn set_children<'a, Ch>(self: &'a Rc<Self>, children: Ch) -> LayoutElements<'a, Ch::Model>
    where
        El: Element,
        Ch: ChildrenNodes,
        Sty: 'static,
        Sc: 'static,
    {
        let children_box = RefMut::map(self.in_cell.borrow_mut(), |x| &mut x.expanded_children);
        LayoutElements::new(
            children,
            children_box,
            &self.global_content,
            self.get_children_layer(&self.in_cell.borrow()),
        )
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
    pub fn global(&self) -> &Rc<GlobalContent> {
        &self.global_content
    }

    /// Get the raw window. Alias to `self.global().window()`.
    pub fn window(&self) -> &WinitWindow {
        self.global_content.window()
    }

    /// Get the region for drawing. Specified by parent element,
    /// and can only be changed by parent element.
    pub fn draw_region(&self) -> Region {
        self.draw_region.get()
    }

    /// Get styles bind to this element
    pub fn styles<Sr>(&self) -> Sr
    where
        Sty: StyleContainer,
        Sr: StyleReader,
    {
        self.in_cell.borrow().styles.read()
    }

    /// Set dirty flag to `true`.
    pub fn set_dirty(self: &Rc<Self>)
    where
        El: Element,
        Sty: 'static,
        Sc: 'static,
    {
        self.global_content.request_redraw(self.clone())
    }

    /// Query whether independent layer was acquired.
    ///
    /// Note that this function will NOT reflect the actual state of
    /// the layer. Independent layer may not exists while result is `true`
    /// and may exists while result is `false`.
    pub fn indep_layer_acquired(&self) -> bool {
        self.acquire_independent_layer.take()
    }

    /// Set `true` to acquire independent render layer for performance optimizations.
    /// Duplicated acquirement will be ignored.
    ///
    /// A independent render layer will cause a heap allocation to render this
    /// element and its children elements rather than drawing on parent's render
    /// layer.
    ///
    /// Recommended when playing animation.
    pub fn acquire_indep_layer(&self, acquire: bool) {
        self.acquire_independent_layer.set(acquire);
        todo!()
    }

    /// Spwan a daemon task on `fut`.
    ///
    /// The spawned task will be cancelled when element dropped,
    /// or can be cancelled manually.
    pub fn daemon<F>(&self, fut: F) -> JoinHandle<Option<F::Output>>
    where
        F: Future + 'static,
    {
        let ed = self.ed.clone();
        tokio::task::spawn_local(async move { ed.cancel_on_abandoned(fut).await })
    }
}

pub struct ElWriteGuard<'a, El, Sty, Sc>
where
    El: Element,
    Sty: 'static,
    Sc: 'static,
{
    write: RwLockMappedWriteGuard<'a, El>,
    set_dirty: &'a RcElementModel<El, Sty, Sc>,
}

impl<El: Element, Sty, Sc> Deref for ElWriteGuard<'_, El, Sty, Sc> {
    type Target = El;
    fn deref(&self) -> &Self::Target {
        &self.write
    }
}

impl<El: Element, Sty, Sc> DerefMut for ElWriteGuard<'_, El, Sty, Sc> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.write
    }
}

impl<El: Element, Sty, Sc> Drop for ElWriteGuard<'_, El, Sty, Sc> {
    fn drop(&mut self) {
        self.set_dirty.set_dirty();
    }
}
