use super::layer::LayerRebuilder;
use crate::{application::event_comp::NewPointerEvent, Result};

pub trait NodeCache: Default + 'static {
    fn render(&self, rebuilder: &mut LayerRebuilder) -> Result<()>;

    // return whether the element is logically entered
    fn emit_event(&mut self, new_event: &NewPointerEvent) -> bool;
}
