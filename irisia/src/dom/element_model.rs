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
};

use super::layer::SharedLayerCompositer;

pub struct ElementModel<Sty, Slt> {
    pub(super) ed: EventDispatcher,
    pub(super) interact_region: Cell<Option<Region>>,
    pub(super) acquire_independent_layer: Cell<bool>,
    pub(super) flag_dirty_setted: Cell<bool>,
    pub(super) global_content: Rc<GlobalContent>,
    pub(super) draw_region: Cell<Region>,
    pub(super) in_cell: RefCell<InsideRefCell<Sty, Slt>>,
}

pub(super) struct InsideRefCell<Sty, Slt> {
    pub parent_layer: Option<Weak<dyn StandaloneRender>>,
    pub event_mgr: NodeEventMgr,
    pub indep_layer: Option<SharedLayerCompositer>,
    pub style: Sty,
    pub slot: Slot<Slt>,
}

pub(super) struct Context<Sty, Slt> {
    pub global_content: Rc<GlobalContent>,
    pub parent_layer: Option<Weak<dyn StandaloneRender>>,
    pub style: Sty,
    pub slot: Slt,
}

impl<Sty, Slt> ElementModel<Sty, Slt> {
    pub(crate) fn new(context: Context) -> Rc<Self> {
        let ed = EventDispatcher::new();

        Rc::new(ElementModel {
            ed: ed.clone(),
            draw_region: Default::default(),
            global_content: context.global_content,
            in_cell: RefCell::new(InsideRefCell {
                parent_layer: context.parent_layer,
                event_mgr: NodeEventMgr::new(ed),
                indep_layer: None,
                style: context.style,
                slot: Slot::new(context.slot),
            }),
            interact_region: Cell::new(None),
            flag_dirty_setted: Cell::new(false),
            acquire_independent_layer: Cell::new(false),
        })
    }
}
