use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    future::Future,
    ops::{Deref, DerefMut},
    rc::{Rc, Weak},
};

use tokio::task::JoinHandle;

use crate::{
    application::{
        content::GlobalContent,
        event_comp::{IncomingPointerEvent, NodeEventMgr},
        redraw_scheduler::StandaloneRender,
    },
    event::{standard::ElementAbandoned, EventDispatcher, Listen},
    primitive::Region,
    structure::StructureCreate,
    style::ReadStyle,
    ElementInterfaces, Result,
};

use super::{
    layer::{LayerCompositer, SharedLayerCompositer},
    ElInputWatcher, SharedEM,
};

#[derive(Clone)]
pub struct ElementAccess(Rc<Shared<dyn ReadStyle>>);

pub struct ElementModel<El> {
    pub(crate) el: RefCell<InitLater<El>>,
    pub(crate) event_mgr: RefCell<NodeEventMgr>,
    pub(crate) shared: Rc<Shared<dyn ReadStyle>>, // TODO: 双Rc优化
}

pub(crate) struct Shared<Sty: ?Sized> {
    pub interact_region: Cell<Option<Region>>,
    pub draw_region: Cell<Region>,
    pub redraw_signal_sent: Cell<bool>,
    pub render_on: RenderOn,
    pub ed: EventDispatcher,
    pub gc: Rc<GlobalContent>,
    pub styles: Sty,
}

pub(crate) enum RenderOn {
    ParentLayer(Weak<dyn StandaloneRender>),
    NewLayer {
        this: Weak<dyn StandaloneRender>,
        layer: SharedLayerCompositer,
    },
}

impl RenderOn {
    fn get_layer(&self) -> &Weak<dyn StandaloneRender> {
        let (Self::NewLayer { this, .. } | Self::ParentLayer(this)) = self;
        this
    }

    pub(crate) fn expect_independent(&self) -> &SharedLayerCompositer {
        match self {
            Self::NewLayer { layer, .. } => layer,
            Self::ParentLayer(_) => {
                panic!("this element did not require a independent layer, it cannot be rendered independently")
            }
        }
    }
}

#[derive(Clone)]
pub struct EMCreateCtx {
    pub(crate) global_content: Rc<GlobalContent>,
    pub(crate) parent_layer: Option<Weak<dyn StandaloneRender>>,
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
            parent_layer: Some(self.0.render_on.get_layer().clone()),
        }
    }

    /// Listen event with options
    pub fn listen<Async, SubEv, WithHd>(
        &self,
    ) -> Listen<EventDispatcher, (), (), Async, SubEv, WithHd> {
        Listen::new(self.event_dispatcher())
    }

    pub fn request_redraw(&self) {
        self.0.request_redraw()
    }

    pub fn styles(&self) -> &dyn ReadStyle {
        &self.0.styles
    }

    pub fn draw_region(&self) -> Region {
        self.0.draw_region.get()
    }
}

impl<Sty: ?Sized> Shared<Sty> {
    fn request_redraw(&self) {
        if self.redraw_signal_sent.get() {
            return;
        }

        self.gc.request_redraw(
            self.render_on
                .get_layer()
                .upgrade()
                .expect("parent rendering layer uninitialized or already dropped"),
        );

        self.redraw_signal_sent.set(true);
    }
}

impl<El> ElementModel<El> {
    pub(crate) fn new<Slt, Sty>(
        context: &EMCreateCtx,
        props: El::Props<'_>,
        styles: Sty,
        slot: Slt,
    ) -> SharedEM<El>
    where
        El: ElementInterfaces,
        Sty: ReadStyle + 'static,
        Slt: StructureCreate,
    {
        let ed = EventDispatcher::new();

        let rc = Rc::new_cyclic(|weak: &Weak<ElementModel<El>>| {
            let render_on = match &context.parent_layer {
                Some(layer) if !El::REQUIRE_INDEPENDENT_LAYER => {
                    RenderOn::ParentLayer(layer.clone())
                }
                _ => RenderOn::NewLayer {
                    this: weak.clone(),
                    layer: LayerCompositer::new(),
                },
            };

            let shared = Rc::new(Shared {
                interact_region: Cell::new(None),
                draw_region: Default::default(),
                redraw_signal_sent: Cell::new(false),
                ed: ed.clone(),
                render_on,
                gc: context.global_content.clone(),
                styles,
            });

            ElementModel {
                el: RefCell::new(InitLater(None)),
                event_mgr: NodeEventMgr::new(ed.clone()).into(),
                shared,
            }
        });

        let new_ctx = match &rc.shared.render_on {
            RenderOn::ParentLayer(_) => Cow::Borrowed(context),
            RenderOn::NewLayer { this, .. } => Cow::Owned(EMCreateCtx {
                global_content: context.global_content.clone(),
                parent_layer: Some(this.clone()),
            }),
        };

        rc.el.borrow_mut().0 = Some(El::create(
            props,
            slot,
            ElementAccess(rc.shared.clone() as _),
            ElInputWatcher::new(Rc::downgrade(&rc)),
            &new_ctx,
        ));

        rc
    }

    pub(crate) fn set_draw_region(&self, region: Region)
    where
        El: ElementInterfaces,
    {
        self.shared.draw_region.set(region);
        self.el.borrow_mut().set_draw_region(region);
    }

    /// Get event dispatcher of this element.
    pub fn event_dispatcher(&self) -> &EventDispatcher {
        &self.shared.ed
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
        &self.shared.gc
    }

    pub fn set_interact_region(&mut self, region: Option<Region>) {
        self.shared.interact_region.set(region);
    }

    pub fn interact_region(&self) -> Option<Region> {
        self.shared.interact_region.get()
    }

    pub fn request_redraw(&self) {
        self.shared.request_redraw()
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

    pub fn access(&self) -> ElementAccess {
        ElementAccess(self.shared.clone())
    }

    /// returns whether this element is logically entered
    pub fn on_pointer_event(&self, ipe: &IncomingPointerEvent) -> bool
    where
        El: ElementInterfaces,
    {
        let children_logically_entered = self.el.borrow_mut().children_emit_event(ipe);
        self.event_mgr.borrow_mut().update_and_emit(
            ipe,
            self.shared.interact_region.get(),
            children_logically_entered,
        )
    }
}

impl<El> Drop for ElementModel<El> {
    fn drop(&mut self) {
        self.shared.ed.emit_trusted(ElementAbandoned)
    }
}

impl<El> StandaloneRender for ElementModel<El>
where
    El: ElementInterfaces,
{
    fn standalone_render(
        &self,
        canvas: &irisia_backend::skia_safe::Canvas,
        interval: std::time::Duration,
    ) -> Result<()> {
        self.shared.redraw_signal_sent.set(false);
        let layer = self.shared.render_on.expect_independent();

        let result = self
            .el
            .borrow_mut()
            .render(&mut LayerCompositer::rebuild(layer, canvas), interval);

        result
    }
}

pub(crate) struct InitLater<T>(pub(super) Option<T>);

impl<T> Deref for InitLater<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

impl<T> DerefMut for InitLater<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap()
    }
}
