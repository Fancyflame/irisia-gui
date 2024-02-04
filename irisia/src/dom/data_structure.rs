use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

use crate::{
    application::{
        content::GlobalContent, event_comp::NodeEventMgr, redraw_scheduler::StandaloneRender,
    },
    event::EventDispatcher,
    primitive::Region,
    structure::Slot,
    Element,
};

use super::{child_nodes::ChildBox, layer::SharedLayerCompositer};

pub struct ElementModel<El: Element, Sty, Slt> {
    pub(super) this: Weak<Self>,
    pub(super) standalone_render: Weak<dyn StandaloneRender>,
    pub(super) el: RefCell<Option<El>>,
    pub(super) ed: EventDispatcher,
    pub(super) draw_region: Cell<Region>,
    pub(super) interact_region: Cell<Option<Region>>,
    pub(super) acquire_independent_layer: Cell<bool>,
    pub(super) flag_dirty_setted: Cell<bool>,
    pub(super) slot: Slot<Slt>,
    pub(super) in_cell: RefCell<InsideRefCell<El, Sty>>,
}

pub(super) struct InsideRefCell<El: Element, Sty> {
    pub children: ChildBox<El::Array>,
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
