use crate::{
    application::event_comp::IncomingPointerEvent,
    dom::{child_nodes::RenderElement, ChildNodes},
    Result,
};

pub use self::props::PropsUpdateWith;
pub use crate::{application::content::GlobalContent, dom::RcElementModel};

pub mod props;

/// Element is a thing can draw itself on the given canvas,
/// according to its properties, styles and given drawing region.
/// This trait is close to the native rendering, if you are not a
/// component maker, please using exist elements or macros to
/// customize one.
pub trait Element
where
    Self: Sized + 'static,
{
    type BlankProps: Default;
    type Slot: ChildNodes;
    type Children: ChildNodes;

    fn render(this: &RcElementModel<Self>, content: RenderElement) -> Result<()>;
    fn on_pointer_event(this: &RcElementModel<Self>, ipe: &IncomingPointerEvent) -> bool;

    fn slot(&self) -> &Self::Slot;
    fn slot_mut(&mut self) -> &mut Self::Slot;
}

pub trait ElementCreate<Pr>: Element + Sized {
    fn el_create(this: &RcElementModel<Self>, props: Pr, slot: Self::Slot) -> Self;
}

pub trait ElementPropsUpdate<Pr>: Element + Sized {
    fn el_update(&mut self, props: Pr);
}

pub trait AsChildren: ChildNodes {}
impl<T: ChildNodes> AsChildren for T {}
