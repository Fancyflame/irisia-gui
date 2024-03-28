use anyhow::anyhow;

use crate::{application::event_comp::IncomingPointerEvent, structure::VisitBy, Result};

use self::layer::{LayerCompositer, LayerRebuilder, SharedLayerCompositer};

pub(crate) use self::element_model::Context as EMCreateCtx;
pub use self::element_model::ElementModel;

mod element_model;
pub(crate) mod layer;
//pub mod pub_handle;

impl ElementModel {
    pub(crate) fn get_or_init_indep_layer(&mut self) -> &SharedLayerCompositer {
        self.indep_layer.get_or_insert_with(LayerCompositer::new)
    }

    pub fn at_new_layer<'a, T>(&'a mut self, base: T) -> Result<LayerRebuilder<'a>>
    where
        T: NewLayerBase<'a>,
    {
        base.create_from(self)
    }

    /// returns whether this element is logically entered
    pub fn emit_event<C>(&mut self, children: &mut C, ipe: &IncomingPointerEvent) -> bool
    where
        C: VisitBy,
    {
        let children_logically_entered = children.emit_event(ipe);
        self.event_mgr
            .update_and_emit(ipe, self.interact_region, children_logically_entered)
    }
}

#[allow(unused)]
fn panic_on_debug(msg: &str) -> Result<()> {
    if cfg!(debug_assertions) {
        panic!("inner error: {}", msg);
    } else {
        Err(anyhow!("{}", msg))
    }
}

pub trait NewLayerBase<'a> {
    fn create_from(self, em: &'a mut ElementModel) -> Result<LayerRebuilder<'a>>;
}

impl<'a> NewLayerBase<'a> for &'a mut LayerRebuilder<'_> {
    fn create_from(self, em: &'a mut ElementModel) -> Result<LayerRebuilder<'a>> {
        self.new_layer(em.get_or_init_indep_layer())
    }
}
