use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

use tokio::sync::RwLock;

use crate::{
    application::{
        content::GlobalContent, event_comp::NodeEventMgr, redraw_scheduler::RedrawObject,
    },
    event::EventDispatcher,
    primitive::Region,
    structure::slot::Slot,
    style::StyleContainer,
    Element,
};

use super::{children::ChildrenBox, layer::SharedLayerCompositer, RenderMultiple};

pub struct ElementModel<El, Sty, Sc>
where
    El: Element,
    Sty: StyleContainer,
    Sc: RenderMultiple,
{
    pub(super) this: Weak<Self>,
    pub(super) el: RwLock<Option<El>>,
    pub(super) global_content: Rc<GlobalContent>,
    pub(super) ed: EventDispatcher,
    pub(super) slot_cache: Slot<Sc>,
    pub(super) draw_region: Cell<Region>,
    pub(super) interact_region: Cell<Option<Region>>,
    pub(super) acquire_independent_layer: Cell<bool>,
    pub(super) in_cell: RefCell<InsideRefCell<Sty>>,
}

pub(super) struct InsideRefCell<Sty> {
    pub styles: Sty,
    pub expanded_children: Option<ChildrenBox>,
    pub event_mgr: NodeEventMgr,
    pub parent_layer: Option<Weak<dyn RedrawObject>>,
    pub indep_layer: Option<SharedLayerCompositer>,
}
