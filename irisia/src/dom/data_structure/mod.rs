use std::sync::{Arc, RwLock as StdRwLock};

use tokio::sync::RwLock;

use crate::{
    application::{content::GlobalContent, event_comp::NodeEventMgr, redraw_scheduler::LayerId},
    event::EventDispatcher,
    primitive::Region,
    structure::slot::Slot,
};

use self::maybe_shared::MaybeShared;

use super::{children::ChildrenBox, layer::LayerCompositer};

pub(super) mod maybe_shared;

pub(super) type RcIndepLayer<El> = maybe_shared::Shared<LayerSharedPart<El>, LayerCompositer>;

pub struct ElementModel<El, Sty, Sc> {
    pub(super) styles: Sty,
    pub(super) slot_cache: Slot<Sc>,
    pub(super) event_mgr: NodeEventMgr,
    pub(super) shared: MaybeShared<LayerSharedPart<El>, LayerCompositer>,
    pub(super) pub_shared: Arc<ElementHandle<El>>,
}

pub(super) struct LayerSharedPart<El> {
    pub(super) pub_shared: Arc<ElementHandle<El>>,
    pub(super) expanded_children: Option<ChildrenBox>,
    pub(super) draw_region: Region,
    pub(super) interact_region: Option<Region>,
}

pub struct ElementHandle<El> {
    pub(super) el: RwLock<Option<El>>,
    pub(super) ed: EventDispatcher,
    pub(super) global_content: Arc<GlobalContent>,
    pub(super) layer_info: StdRwLock<LayerInfo>,
}

pub(super) struct LayerInfo {
    pub acquire_independent_layer: bool,
    pub parent_layer_id: LayerId,
    pub indep_layer_id: Option<LayerId>,
}

impl LayerInfo {
    pub fn render_layer_id(&self) -> LayerId {
        self.indep_layer_id.unwrap_or(self.parent_layer_id)
    }
}
