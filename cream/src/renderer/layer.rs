use getset::{CopyGetters, Getters, MutGetters};

use crate::{event::event_target::EventTarget, primary::Area, structure::Element};

#[derive(Getters, MutGetters)]
pub struct Layer<E>
where
    E: Element,
{
    #[getset(get = "pub", get_mut)]
    ele: E,

    style: E::Style,
    attrs: LayerAttribute,
}

#[derive(Getters, MutGetters, CopyGetters)]
pub(crate) struct LayerAttribute {
    #[getset(get = "pub", get_mut = "pub(crate)")]
    graphic: Vec<u8>,

    #[getset(get_copy = "pub", get_mut = "pub(crate)")]
    area: Area,

    #[getset(get = "pub", get_mut = "pub")]
    event_target: EventTarget,
}

pub(crate) trait LayerTrait {
    fn attrs(&self) -> &LayerAttribute;
}

impl<E: Element> LayerTrait for Layer<E> {
    fn attrs(&self) -> &LayerAttribute {
        &self.attrs
    }
}
