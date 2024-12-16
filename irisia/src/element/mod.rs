use std::time::Duration;

use crate::{
    application::event_comp::IncomingPointerEvent,
    el_model::{EMCreateCtx, ElementAccess},
    hook::ProviderObject,
    model::{iter::ModelMapper, DesiredVModel},
    Result,
};

pub use component::{Component, ComponentTemplate};
use deps::AsEmptyProps;
use irisia_backend::skia_safe::{Canvas, Region as SkRegion};

pub mod children_utils;
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
    type Props: AsEmptyProps;
    type AcceptChild: ModelMapper;
    const REQUIRE_INDEPENDENT_LAYER: bool = false;

    fn create<Slt>(
        props: &Self::Props,
        access: ElementAccess,
        slot: ProviderObject<Slt>,
        ctx: &EMCreateCtx,
    ) -> Self
    where
        Slt: DesiredVModel<Self::AcceptChild> + 'static;

    fn render(&mut self, args: Render) -> Result<()>;
    fn spread_event(&mut self, ipe: &IncomingPointerEvent) -> bool;
    fn on_draw_region_change(&mut self);
}
