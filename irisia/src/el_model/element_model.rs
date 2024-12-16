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
    hook::ProviderObject,
    model::DesiredVModel,
    primitive::Region,
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
    layouting_draw_region: Option<Region>,
}

pub(crate) struct Shared {
    interact_region: Cell<Region>,
    last_draw_region: Cell<Option<Region>>,
    draw_region: Cell<Option<Region>>,
    redraw_signal_sent: Cell<bool>,
    ed: EventDispatcher,
    gc: Rc<GlobalContent>,
}

#[derive(Clone)]
pub struct EMCreateCtx {
    pub(crate) global_content: Rc<GlobalContent>,
}

impl<El, Cp> ElementModel<El, Cp> {
    pub fn child_data(&self) -> &Cp {
        &self.child_props
    }

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
        props: &El::Props,
        child_props: Cp,
        slot: ProviderObject<Slt>,
    ) -> Self
    where
        Slt: DesiredVModel<El::AcceptChild>,
    {
        let ed = EventDispatcher::new();

        let access = ElementAccess(Rc::new(Shared {
            interact_region: Cell::new(Region::default()),
            last_draw_region: Cell::new(None),
            draw_region: Default::default(),
            redraw_signal_sent: Cell::new(false),
            ed: ed.clone(),
            gc: context.global_content.clone(),
        }));

        ElementModel {
            el: El::create(props, access.clone(), slot, &context),
            event_mgr: NodeEventMgr::new(ed.clone()).into(),
            redraw_hook: {
                let access = access.clone();
                Observer::new(move || {
                    access.request_redraw();
                    true
                })
            },
            access,
            child_props,
            layouting_draw_region: None,
        }
    }

    pub(crate) fn set_draw_region(&mut self, region: Option<Region>) {
        if region == self.access.0.draw_region.get() {
            return;
        }
        self.access.0.draw_region.set(region);
        self.el.on_draw_region_change();
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

    pub fn render(&mut self, args: Render) -> Result<()> {
        let Some(draw_region) = self.access.draw_region() else {
            return Ok(());
        };

        if !args
            .dirty_region
            .intersects_rect(draw_region.ceil_to_irect())
        {
            return Ok(());
        }

        self.access.0.redraw_signal_sent.set(false);
        self.redraw_hook.invoke(|| self.el.render(args))
    }
}

impl<El, Cp> Drop for ElementModel<El, Cp> {
    fn drop(&mut self) {
        self.event_dispatcher().emit_trusted(ElementAbandoned)
    }
}

impl ElementAccess {
    pub fn interact_region(&self) -> Region {
        self.0.interact_region.get()
    }

    pub fn set_interact_region(&self, region: Region) {
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

    pub(crate) fn reset_redraw_region_pair(&self) -> [Option<Region>; 2] {
        let mut old_region = self.0.last_draw_region.get();
        let new_region = self.draw_region();

        if old_region == new_region {
            old_region = None;
        } else {
            self.0.last_draw_region.set(new_region);
        }

        [old_region, new_region]
    }

    pub fn draw_region(&self) -> Option<Region> {
        self.0.draw_region.get()
    }
}
