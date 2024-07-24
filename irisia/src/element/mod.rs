use std::time::Duration;

use crate::{
    application::event_comp::IncomingPointerEvent,
    el_model::{layer::LayerRebuilder, EMCreateCtx, ElInputWatcher, ElementAccess},
    primitive::Region,
    structure::StructureCreate,
    Result,
};

pub use component::{CompInputWatcher, Component, ComponentTemplate, OneStructureCreate};

mod component;

#[doc(hidden)]
pub mod macro_helper;

#[derive(Clone, Copy, Default)]
pub struct EmptyProps {}

/// Element is a thing can draw itself on the given canvas,
/// according to its properties, styles and given drawing region.
/// This trait is close to the native rendering, if you are not a
/// component maker, please using exist elements or macros to
/// customize one.
pub trait ElementInterfaces: Sized + 'static {
    type Props<'a>: Default;
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

pub trait UserProps {
    type Props;
    fn take(props: Self::Props) -> Self;
}

pub enum FieldMustInit<T> {
    NotInit { field_name: &'static str },
    Init { value: T },
}

impl<T> FieldMustInit<T> {
    pub const fn new_uninit(field_name: &'static str) -> Self {
        Self::NotInit { field_name }
    }

    pub fn take(self) -> T {
        match self {
            Self::Init { value } => value,
            Self::NotInit { field_name } => {
                panic!("field `{field_name}` of this props must be initialized")
            }
        }
    }
}

impl<T> From<T> for FieldMustInit<T> {
    fn from(value: T) -> Self {
        Self::Init { value }
    }
}
