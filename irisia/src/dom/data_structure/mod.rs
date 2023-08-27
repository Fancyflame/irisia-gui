use std::{
    ops::Deref,
    sync::{Arc, RwLock as StdRwLock},
};

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
    pub(super) pub_shared: FullElementHandle<El>,
}

pub(super) struct LayerSharedPart<El> {
    pub(super) pub_shared: FullElementHandle<El>,
    pub(super) expanded_children: Option<ChildrenBox>,
    pub(super) draw_region: Region,
    pub(super) interact_region: Option<Region>,
}

pub struct FullElementHandle<El> {
    pub(super) el: Arc<RwLock<Option<El>>>,
    pub(super) base: Arc<ElementHandle>,
}

pub struct ElementHandle {
    pub(super) ed: EventDispatcher,
    pub(super) global_content: Arc<GlobalContent>,
    pub(super) layer_info: StdRwLock<LayerInfo>,
}

pub(super) struct LayerInfo {
    pub acquire_independent_layer: bool,
    pub parent_layer_id: LayerId,
    pub indep_layer_id: Option<LayerId>,
}

impl<El> Clone for FullElementHandle<El> {
    fn clone(&self) -> Self {
        Self {
            el: self.el.clone(),
            base: self.base.clone(),
        }
    }
}

impl<El> Deref for FullElementHandle<El> {
    type Target = Arc<ElementHandle>;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl LayerInfo {
    pub fn render_layer_id(&self) -> LayerId {
        self.indep_layer_id.unwrap_or(self.parent_layer_id)
    }
}
