use std::time::Duration;

use crate::{
    application::event_comp::IncomingPointerEvent,
    data_flow::{const_wire, ReadWire},
    el_model::{EMCreateCtx, ElementAccess},
    structure::StructureCreate,
    Result,
};

pub use component::{Component, ComponentTemplate, RootStructureCreate};
use deps::AsEmptyProps;
use irisia_backend::skia_safe::{Canvas, Region as SkRegion};

mod component;
pub mod deps;

#[derive(Clone, Copy)]
pub struct Render<'a> {
    pub canvas: &'a Canvas,
    pub interval: Duration,
    pub dirty_region: &'a SkRegion,
}

/// Element is a thing can draw itself on the given canvas,
/// according to its properties, styles and given drawing region.
/// This trait is close to the native rendering, if you are not a
/// component maker, please using exist elements or macros to
/// customize one.
pub trait ElementInterfaces: Sized + 'static {
    type Props<'a>: AsEmptyProps;
    type SlotData: 'static;
    const REQUIRE_INDEPENDENT_LAYER: bool = false;

    fn create<Slt>(
        props: Self::Props<'_>,
        slot: Slt,
        access: ElementAccess,
        ctx: &EMCreateCtx,
    ) -> Self
    where
        Slt: StructureCreate<Self::SlotData>;

    fn render(&mut self, args: Render) -> Result<()>;
    fn spread_event(&mut self, ipe: &IncomingPointerEvent) -> bool;
    fn on_draw_region_changed(&mut self);
}
