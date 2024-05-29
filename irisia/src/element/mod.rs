use std::time::Duration;

use crate::{
    application::event_comp::IncomingPointerEvent,
    el_model::{layer::LayerRebuilder, EMCreateCtx, ElInputWatcher, ElementAccess},
    primitive::Region,
    structure::StructureCreate,
    Result,
};

pub use component::{CompInputWatcher, Component, ComponentTemplate};

mod component;

/// Element is a thing can draw itself on the given canvas,
/// according to its properties, styles and given drawing region.
/// This trait is close to the native rendering, if you are not a
/// component maker, please using exist elements or macros to
/// customize one.
pub trait ElementInterfaces: Sized + 'static {
    type Props<'a>;
    const REQUIRE_INDEPENDENT_LAYER: bool = false;

    fn create<Slt>(
        props: Self::Props<'_>,
        slot: Slt,
        access: ElementAccess,
        watch_input: ElInputWatcher<Self>,
        ctx: &EMCreateCtx,
    ) -> Self
    where
        Slt: StructureCreate;

    fn render(&mut self, lr: &mut LayerRebuilder, interval: Duration) -> Result<()>;
    fn set_draw_region(&mut self, dr: Region);
    fn children_emit_event(&mut self, ipe: &IncomingPointerEvent) -> bool;
}
