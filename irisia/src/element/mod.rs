use std::time::Duration;

use crate::{
    dom::{layer::LayerRebuilder, ElementModel},
    Result, StyleReader,
};

pub use crate::{application::content::GlobalContent, dom::RcElementModel};

/// Element is a thing can draw itself on the given canvas,
/// according to its properties, styles and given drawing region.
/// This trait is close to the native rendering, if you are not a
/// component maker, please using exist elements or macros to
/// customize one.
pub trait Element
where
    Self: Sized + 'static,
{
    type Style;
    type Slot;

    fn render(&mut self, lr: &mut LayerRebuilder, interval: Duration) -> Result<()>;
    fn element_model(&self) -> &ElementModel<Self::Style, Self::Slot>;

    fn on_style_update(&mut self) {}
    fn on_slot_update(&mut self) {}
    fn on_draw_region_update(&mut self) {}
}
