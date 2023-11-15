use std::{future::Future, rc::Rc};
use tokio::{
    sync::{RwLockMappedWriteGuard, RwLockReadGuard, RwLockWriteGuard},
    task::JoinHandle,
};

use crate::{
    application::content::GlobalContent,
    event::{standard::ElementAbandoned, EdProvider, EventDispatcher, Listen},
    primitive::Region,
    style::StyleContainer,
    Element, Result, StyleReader,
};

use self::write_guard::ElWriteGuard;

use super::{
    data_structure::{Context, ElementModel},
    RcElementModel,
};

mod write_guard;

const TRY_ACCESS_ERROR: &str = "do not hold a element write guard across `await`. \
                this limitation will be lifted in the future, but still \
                discouraged";

impl<El, Sty> ElementModel<El, Sty>
where
    El: Element,
    Sty: StyleContainer + 'static,
{
    /// Get a write guard without setting dirty
    pub(super) fn el_write_clean(&self) -> RwLockMappedWriteGuard<El> {
        RwLockWriteGuard::map(self.el.try_write().expect(TRY_ACCESS_ERROR), |x| {
            x.as_mut().unwrap()
        })
    }

    /// Get a write guard of this element and setting dirty.
    /// `None` if this element will no longer used.
    pub async fn el_write<'a>(self: &'a Rc<Self>) -> Option<ElWriteGuard<'a, El, Rc<Self>>> {
        RwLockWriteGuard::try_map(self.el.write().await, |x| x.as_mut())
            .ok()
            .map(|guard| ElWriteGuard {
                write: guard,
                set_dirty: self,
            })
    }

    /// Get a read guard of this element and dirty flag is not affected.
    /// `None` if this element will no longer used.
    pub async fn el_read(&self) -> Option<RwLockReadGuard<El>> {
        RwLockReadGuard::try_map(self.el.read().await, |x| x.as_ref()).ok()
    }

    pub fn attached(&self) -> bool {
        matches!(self.in_cell.borrow().context, Context::Attached(_))
    }

    /// Listen event with options
    pub fn listen<'a>(self: &'a Rc<Self>) -> Listen<'a, Rc<Self>, (), (), (), ()> {
        Listen::new(self)
    }

    /// Get event dispatcher of this element.
    pub fn event_dispatcher(&self) -> &EventDispatcher {
        &self.ed
    }

    /// Let this element being focused on.
    pub fn focus(&self) {
        if let Ok(c) = self.in_cell.borrow().ctx() {
            c.global_content.focusing().focus(self.ed.clone());
        }
    }

    /// Let this element no longer being focused. does nothing if
    /// this element is not in focus.
    pub fn blur(&self) {
        if let Ok(c) = self.in_cell.borrow().ctx() {
            c.global_content.focusing().blur_checked(&self.ed);
        }
    }

    /// Get global content of the window.
    pub fn global_content(&self) -> Result<Rc<GlobalContent>> {
        self.in_cell
            .borrow()
            .ctx()
            .map(|c| c.global_content.clone())
    }

    /// Get the region for drawing. Specified by parent element,
    /// and can only be changed by parent element.
    pub fn draw_region(&self) -> Region {
        self.draw_region.get()
    }

    pub fn set_interact_region(&self, region: Option<Region>) {
        if region == self.interact_region.get() {
            return;
        }
        self.interact_region.set(region);
    }

    pub fn interact_region(&self) -> Option<Region> {
        self.interact_region.get()
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
    pub fn set_dirty(&self) {
        if let Ok(c) = self.in_cell.borrow().ctx() {
            c.global_content.request_redraw(
                self.get_children_layer(&self.in_cell.borrow())
                    .upgrade()
                    .expect("parent layer unexpectedly dropped"),
            )
        }
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
        if self.indep_layer_acquired() == acquire {
            return;
        }
        self.acquire_independent_layer.set(acquire);
        self.set_dirty();
    }

    /// Spwan a daemon task on `fut`.
    ///
    /// The spawned task will be cancelled when element dropped,
    /// or can be cancelled manually.
    pub fn daemon<F>(&self, fut: F) -> JoinHandle<()>
    where
        F: Future + 'static,
    {
        let ed = self.ed.clone();
        tokio::task::spawn_local(async move {
            tokio::select! {
                _ = ed.recv_trusted::<ElementAbandoned>() => {},
                _ = fut => {}
            }
        })
    }
}

impl<El, Sty> EdProvider for RcElementModel<El, Sty>
where
    El: Element,
    Sty: StyleContainer + 'static,
{
    fn event_dispatcher(&self) -> &EventDispatcher {
        ElementModel::event_dispatcher(self)
    }

    fn daemon<F>(&self, f: F) -> JoinHandle<()>
    where
        F: Future + 'static,
    {
        ElementModel::daemon(self, f)
    }

    fn handle_available(&self) -> bool {
        self.attached()
    }
}
