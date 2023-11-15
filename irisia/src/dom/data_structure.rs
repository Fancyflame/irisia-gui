use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

use tokio::sync::RwLock;

use crate::{
    application::{
        content::GlobalContent, event_comp::NodeEventMgr, redraw_scheduler::StandaloneRender,
    },
    event::EventDispatcher,
    primitive::Region,
    style::StyleContainer,
    Element,
};

use super::layer::SharedLayerCompositer;

pub struct ElementModel<El, Sty>
where
    El: Element,
    Sty: StyleContainer,
{
    pub(super) this: Weak<Self>,
    pub(super) el: RwLock<Option<El>>,
    pub(super) ed: EventDispatcher,
    pub(super) draw_region: Cell<Region>,
    pub(super) interact_region: Cell<Option<Region>>,
    pub(super) acquire_independent_layer: Cell<bool>,
    pub(super) in_cell: RefCell<InsideRefCell<Sty>>,
}

pub(super) struct InsideRefCell<Sty> {
    pub styles: Sty,
    pub event_mgr: NodeEventMgr,
    pub context: Context,
    pub indep_layer: Option<SharedLayerCompositer>,
}

pub(super) enum Context {
    None,
    Attached(AttachedCtx),
    Destroyed,
}

pub(super) struct AttachedCtx {
    pub global_content: Rc<GlobalContent>,
    pub parent_layer: Option<Weak<dyn StandaloneRender>>,
}
