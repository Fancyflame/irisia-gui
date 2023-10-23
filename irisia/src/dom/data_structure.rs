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
    style::StyleContainer,
    Element,
};

use super::{layer::SharedLayerCompositer, ChildNodes};

pub struct ElementModel<El, Sty, Sc>
where
    El: Element,
    Sty: StyleContainer,
    Sc: ChildNodes,
{
    pub(super) this: Weak<Self>,
    pub(super) el: RwLock<Option<El>>,
    pub(super) ed: EventDispatcher,
    pub(super) draw_region: Cell<Region>,
    pub(super) interact_region: Cell<Option<Region>>,
    pub(super) acquire_independent_layer: Cell<bool>,
    pub(super) in_cell: RefCell<InsideRefCell<Sty, El::Children<Sc>>>,
}

pub(super) struct InsideRefCell<Sty, Ch> {
    pub styles: Sty,
    pub expanded_children: Ch,
    pub event_mgr: NodeEventMgr,
    pub context: Context,
    pub indep_layer: Option<SharedLayerCompositer>,
}

pub(super) enum Context {
    None,
    Attached {
        global_content: Rc<GlobalContent>,
        parent_layer: Option<Weak<dyn RedrawObject>>,
    },
    Destroyed,
}
