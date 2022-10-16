use std::rc::Rc;

use getset::{CopyGetters, Getters, MutGetters};

use crate::{
    data_driven::MutData, event::event_target::EventTarget, primary::Area, style::StyleTable,
};

use super::element::ElementHandle;

#[derive(Getters, MutGetters, CopyGetters)]
pub struct ElemService {
    #[getset(get = "pub")]
    area: Rc<MutData<Area>>,

    #[getset(get = "pub", get_mut = "pub(crate)")]
    children: Vec<ElementHandle>,

    #[getset(get_copy = "pub(crate)", get_mut = "pub(crate)")]
    renderer_index: Option<usize>,

    #[getset(get = "pub", get_mut = "pub(crate)")]
    event_target: EventTarget,

    #[getset(get = "pub", get_mut = "pub(crate)")]
    style: StyleTable,
}

impl ElemService {
    pub fn new() -> Self {
        ElemService {
            area: MutData::new(<_>::default()),
            children: Vec::new(),
            renderer_index: None,
            event_target: EventTarget::new(),
            style: StyleTable::new(),
        }
    }

    pub(crate) fn reset(&mut self) {
        self.children.clear();
        self.renderer_index.take();
        self.event_target.clear();
        self.style.clear();
    }

    pub fn distribute_area(&self, area: Area) {
        self.area.set(area);
    }
}
