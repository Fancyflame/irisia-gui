use std::time::Duration;

use crate::{
    application::event_comp::IncomingPointerEvent,
    data_flow::{const_wire, ReadWire},
    el_model::{EMCreateCtx, ElementAccess},
    structure::StructureCreate,
    Result,
};

pub use component::{Component, ComponentTemplate, RootStructureCreate};
use irisia_backend::skia_safe::{Canvas, Region as SkRegion};

mod component;

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
    type Props<'a>: FromUserProps;
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

pub trait FromUserProps {
    type Props: Default;
    fn take(props: Self::Props) -> Self;
}

pub enum FieldPlaceholder<T> {
    MustInit { field_name: &'static str },
    OrDefault(fn() -> T),
    Optioned,
    Init(ReadWire<T>),
}

impl<T: 'static> FieldPlaceholder<T> {
    pub const fn initialized(value: ReadWire<T>) -> Self {
        Self::Init(value)
    }

    pub fn take(self) -> ReadWire<T> {
        match self {
            Self::Init(value) => value,
            Self::OrDefault(default_with) => const_wire(default_with()),
            Self::MustInit { field_name } => {
                panic!("field `{field_name}` of this props must be initialized")
            }
            Self::Optioned => panic!("cannot call `take` on `FieldPlaceholder::Optioned`"),
        }
    }

    pub fn take_optioned(self) -> Option<ReadWire<T>> {
        match self {
            Self::Init(value) => Some(value),
            Self::OrDefault(default_with) => Some(const_wire(default_with())),
            Self::MustInit { .. } | Self::Optioned => None,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct EmptyProps {}

impl FromUserProps for () {
    type Props = EmptyProps;
    fn take(_: Self::Props) -> Self {}
}
