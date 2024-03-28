use std::{
    cell::Cell,
    future::Future,
    rc::{Rc, Weak},
};

use anyhow::anyhow;
use tokio::task::JoinHandle;

use crate::{
    application::{
        content::GlobalContent, event_comp::NodeEventMgr, redraw_scheduler::StandaloneRender,
    },
    event::{standard::ElementAbandoned, EventDispatcher, Listen},
    primitive::Region,
    Result,
};

use super::layer::SharedLayerCompositer;

pub struct ElementModel {
    pub(super) ed: EventDispatcher,
    pub(super) interact_region: Option<Region>,
    pub(super) flag_dirty_setted: Cell<bool>,
    pub(super) global_content: Rc<GlobalContent>,
    pub(super) parent_layer: Weak<dyn StandaloneRender>,
    pub(super) event_mgr: NodeEventMgr,
    pub(super) indep_layer: Option<SharedLayerCompositer>,
}

pub struct Context {
    pub(crate) global_content: Rc<GlobalContent>,
    pub(crate) parent_layer: Weak<dyn StandaloneRender>,
}

impl ElementModel {
    pub(crate) fn new(context: &Context) -> Self {
        let ed = EventDispatcher::new();

        ElementModel {
            ed: ed.clone(),
            global_content: context.global_content.clone(),
            parent_layer: context.parent_layer.clone(),
            event_mgr: NodeEventMgr::new(ed),
            indep_layer: None,
            interact_region: None,
            flag_dirty_setted: Cell::new(false),
        }
    }

    /// Listen event with options
    pub fn listen<'a, Async, SubEv, WithHd>(
        self: &'a Rc<Self>,
    ) -> Listen<'a, Rc<Self>, (), (), Async, SubEv, WithHd> {
        Listen::new(self)
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
    pub fn global_content(&self) -> &Rc<GlobalContent> {
        &self.global_content
    }

    pub fn set_interact_region(&mut self, region: Option<Region>) {
        self.interact_region = region;
    }

    pub fn interact_region(&self) -> Option<Region> {
        self.interact_region
    }

    pub fn request_redraw(&self) -> Result<()> {
        if self.flag_dirty_setted.get() {
            return Ok(());
        }

        self.global_content.request_redraw(
            self.parent_layer.upgrade().ok_or_else(|| {
                anyhow!("parent rendering layer uninitialized or already dropped")
            })?,
        );

        self.flag_dirty_setted.set(true);
        Ok(())
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

impl Drop for ElementModel {
    fn drop(&mut self) {
        self.ed.emit_trusted(ElementAbandoned)
    }
}
