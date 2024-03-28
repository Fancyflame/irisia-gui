use std::{future::Future, rc::Rc};
use tokio::{sync::TryLockError, task::JoinHandle};

use crate::{
    application::content::GlobalContent,
    event::{standard::ElementAbandoned, EdProvider, EventDispatcher, Listen},
    primitive::Region,
    Element, Result, StyleReader,
};

use super::element_model::ElementModel;

pub type TryLockResult<T> = std::result::Result<T, TryLockError>;

impl<Sty, Slt> ElementModel<Sty, Slt> {
    pub fn update_slot<F>(&self, update: F)
    where
        F: for<'a> FnOnce(&'a mut Slt),
        Slt: 'static,
    {
        self.slot.update(update)
    }

    pub fn attached(&self) -> bool {
        self.in_cell.borrow().context.is_some()
    }

    /// Get the region for drawing. Specified by parent element,
    /// and can only be changed by parent element.
    pub fn draw_region(&self) -> Region {
        self.draw_region.get()
    }

    /// Get styles bind to this element
    pub fn styles<Sr>(&self) -> Sr
    where
        Sty: StyleGroup,
        Sr: StyleReader,
    {
        self.in_cell.borrow().styles.read()
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
        self.request_redraw();
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

impl<El, Sty, Slt> EdProvider for RcElementModel<El, Sty, Slt>
where
    Self: 'static,
    El: Element,
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
