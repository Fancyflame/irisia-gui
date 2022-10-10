use std::cell::RefCell;

use anyhow::Result;
use getset::{CopyGetters, Getters, MutGetters, Setters};
use smallvec::SmallVec;

use crate::{
    event::event_target::EventTarget, map_rc::MapRc, primary::Area, structure::Element,
    style::StyleTable,
};

use super::renderer::Renderer;

pub type RcLayer<E> = MapRc<RefCell<Layer<E>>>;

#[derive(Getters, MutGetters, Setters)]
pub struct Layer<E>
where
    E: Element,
{
    #[getset(get = "pub", set = "pub")]
    ele_cache: RefCell<E>,

    children: SmallVec<[MapRc<RefCell<E::AcceptChildren>>; 10]>,

    attrs: LayerAttribute,
}

impl<E: Element> Layer<E> {
    pub fn new(ele: E) -> Self {
        Layer {
            ele_cache: RefCell::new(ele),
            children: SmallVec::new(),
            attrs: LayerAttribute::new(),
        }
    }
}

#[derive(Getters, MutGetters, CopyGetters)]
pub(crate) struct LayerAttribute {
    #[getset(get = "pub", get_mut = "pub(crate)")]
    graphic: Vec<u8>,

    #[getset(get_copy = "pub", get_mut = "pub(crate)")]
    area: Area,

    #[getset(get = "pub", get_mut = "pub")]
    event_target: EventTarget,

    #[getset(get = "pub", get_mut = "pub(crate)")]
    renderer_index: Option<usize>,

    #[getset(get = "pub", get_mut = "pub")]
    style_table: StyleTable,
}

impl LayerAttribute {
    fn new() -> Self {
        LayerAttribute {
            graphic: Vec::new(),
            area: <_>::default(),
            event_target: EventTarget::new(),
            renderer_index: None,
            style_table: StyleTable::new(),
        }
    }
}

pub(crate) trait LayerTrait {
    fn render(&mut self, r: &mut Renderer) -> Result<()>;
    fn attrs(&self) -> &LayerAttribute;
}

impl<E: Element> LayerTrait for Layer<E> {
    fn render(&mut self, r: &mut Renderer) -> Result<()> {
        self.ele_cache
            .borrow_mut()
            .render(r.canvas(), self.attrs.style_table(), todo!())
    }

    fn attrs(&self) -> &LayerAttribute {
        &self.attrs
    }
}
