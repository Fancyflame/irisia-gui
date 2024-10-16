use std::{cell::Cell, future::Future, rc::Rc};

use tokio::task::JoinHandle;

use crate::{
    application::{
        content::GlobalContent,
        event_comp::{IncomingPointerEvent, NodeEventMgr},
    },
    data_flow::observer::Observer,
    element::Render,
    event::{standard::ElementAbandoned, EventDispatcher, Listen},
    primitive::Region,
    structure::StructureCreate,
    ElementInterfaces, Result,
};

#[derive(Clone)]
pub struct ElementAccess(Rc<Shared>);

pub struct ElementModel<El, Cp> {
    pub(crate) el: El,
    pub(crate) child_props: Cp,
    event_mgr: NodeEventMgr,
    access: ElementAccess,
    redraw_hook: Observer,
    dirty: bool,
}

pub(crate) struct Shared {
    interact_region: Cell<Option<Region>>,
    last_draw_region: Cell<Option<Region>>,
    draw_region: Cell<Region>,
    redraw_signal_sent: Cell<bool>,
    ed: EventDispatcher,
    gc: Rc<GlobalContent>,
}

#[derive(Clone)]
pub struct EMCreateCtx {
    pub(crate) global_content: Rc<GlobalContent>,
}

impl<El, Cp> ElementModel<El, Cp> {
    /// Get event dispatcher of this element.
    pub fn event_dispatcher(&self) -> &EventDispatcher {
        self.access.event_dispatcher()
    }

    /// Let this element being focused on.
    pub fn focus(&self) {
        self.global_content()
            .focusing()
            .focus(self.event_dispatcher().clone());
    }

    /// Let this element no longer being focused. does nothing if
    /// this element is not in focus.
    pub fn blur(&self) {
        self.global_content()
            .focusing()
            .blur_checked(&self.event_dispatcher());
    }

    /// Get global content of the window.
    pub fn global_content(&self) -> &Rc<GlobalContent> {
        self.access.global_content()
    }

    pub fn request_redraw(&self) {
        self.access.request_redraw()
    }

    /// Spwan a daemon task on `fut`.
    ///
    /// The spawned task will be cancelled when element dropped,
    /// or can be cancelled manually.
    pub fn daemon<F>(&self, fut: F) -> JoinHandle<()>
    where
        F: Future + 'static,
    {
        let ed = self.event_dispatcher().clone();
        tokio::task::spawn_local(async move {
            tokio::select! {
                _ = ed.recv_trusted::<ElementAbandoned>() => {},
                _ = fut => {}
            }
        })
    }

    pub fn access(&self) -> &ElementAccess {
        &self.access
    }
}

impl<El, Cp> ElementModel<El, Cp>
where
    El: ElementInterfaces,
{
    pub(crate) fn new<Slt>(
        context: &EMCreateCtx,
        props: El::Props<'_>,
        child_props: Cp,
        slot: Slt,
    ) -> Self
    where
        Slt: StructureCreate,
    {
        let ed = EventDispatcher::new();

        let access = ElementAccess(Rc::new(Shared {
            interact_region: Cell::new(None),
            last_draw_region: Cell::new(None),
            draw_region: Default::default(),
            redraw_signal_sent: Cell::new(false),
            ed: ed.clone(),
            gc: context.global_content.clone(),
        }));

        ElementModel {
            el: El::create(props, slot, access.clone(), &context),
            event_mgr: NodeEventMgr::new(ed.clone()).into(),
            dirty: true,
            redraw_hook: {
                let access = access.clone();
                Observer::new(move || {
                    access.request_redraw();
                    false
                })
            },
            access,
            child_props,
        }
    }

    pub(crate) fn set_draw_region(&mut self, region: Region) {
        if region == self.access.0.draw_region.get() {
            return;
        }
        self.access.0.draw_region.set(region);
        self.el.on_draw_region_changed();
    }

    /// returns whether this element is logically entered
    pub fn on_pointer_event(&mut self, ipe: &IncomingPointerEvent) -> bool {
        let children_logically_entered = self.el.spread_event(ipe);
        self.event_mgr.update_and_emit(
            ipe,
            self.access.interact_region(),
            children_logically_entered,
        )
    }

    pub fn check_mark_dirty(&mut self, dirty_region: Region) {
        if !self.access.draw_region().intersects(dirty_region) {
            return;
        }

        self.dirty = true;
        self.el.spread_mark_dirty(dirty_region);
    }

    pub fn render(&mut self, args: Render) -> Result<()> {
        if !self.dirty {
            return Ok(());
        }

        self.access.0.redraw_signal_sent.set(false);
        let result = self.redraw_hook.invoke(|| self.el.render(args));
        if result.is_ok() {
            self.dirty = false;
        }
        result
    }
}

impl<El, Cp> Drop for ElementModel<El, Cp> {
    fn drop(&mut self) {
        self.event_dispatcher().emit_trusted(ElementAbandoned)
    }
}

impl ElementAccess {
    pub fn interact_region(&self) -> Option<Region> {
        self.0.interact_region.get()
    }

    pub fn set_interact_region(&self, region: Option<Region>) {
        self.0.interact_region.set(region)
    }

    pub fn event_dispatcher(&self) -> &EventDispatcher {
        &self.0.ed
    }

    pub fn global_content(&self) -> &Rc<GlobalContent> {
        &self.0.gc
    }

    pub fn context(&self) -> EMCreateCtx {
        EMCreateCtx {
            global_content: self.0.gc.clone(),
        }
    }

    /// Listen event with options
    pub fn listen<Async, SubEv, WithHd>(
        &self,
    ) -> Listen<EventDispatcher, (), (), Async, SubEv, WithHd> {
        Listen::new(self.event_dispatcher())
    }

    pub fn request_redraw(&self) {
        if self.0.redraw_signal_sent.get() {
            return;
        }

        self.0.gc.request_redraw(self.clone());
        self.0.redraw_signal_sent.set(true);
    }

    pub(crate) fn reset_redraw_region_pair(&self) -> (Option<Region>, Region) {
        let new_region = self.0.draw_region.get();
        let old_region = self
            .0
            .last_draw_region
            .get()
            .take_if(|old| *old != new_region);
        self.0.last_draw_region.set(Some(new_region));
        (old_region, new_region)
    }

    pub fn draw_region(&self) -> Region {
        self.0.draw_region.get()
    }
}
