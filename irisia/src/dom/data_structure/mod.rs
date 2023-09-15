use std::{
    cell::{Cell, RefCell},
    sync::Arc,
};

use tokio::sync::RwLock;

use crate::{
    application::{content::GlobalContent, event_comp::NodeEventMgr},
    event::EventDispatcher,
    primitive::Region,
    structure::slot::Slot,
};

use super::{
    children::ChildrenBox,
    layer::{SharedLayerCompositer, WeakLayerCompositer},
};

pub(super) mod maybe_shared;

pub struct ElementModel<El, Sty, Sc> {
    pub(super) el: RwLock<Option<El>>,
    pub(super) global_content: Arc<GlobalContent>,
    pub(super) ed: EventDispatcher,
    pub(super) slot_cache: Slot<Sc>,
    pub(super) draw_region: Cell<Region>,
    pub(super) interact_region: Cell<Option<Region>>,
    pub(super) acquire_independent_layer: Cell<bool>,
    pub(super) in_cell: RefCell<InsideRefCell<Sty>>,
}

pub(super) struct InsideRefCell<Sty> {
    pub(super) styles: Sty,
    pub(super) expanded_children: Option<ChildrenBox>,
    pub(super) event_mgr: NodeEventMgr,
    pub(super) parent_layer: WeakLayerCompositer,
    pub(super) indep_layer: Option<SharedLayerCompositer>,
}

impl<Sty> InsideRefCell<Sty> {
    pub(super) fn get_children_layer(&self) -> SharedLayerCompositer {
        match self.indep_layer {
            Some(il) => il.clone(),
            None => self.parent_layer.upgrade().expect("parent layer dropped"),
        }
    }
}
