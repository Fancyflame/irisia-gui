use crate::{
    application::event_comp::IncomingPointerEvent,
    dom::{child_nodes::RenderElement, ChildNodes},
    Result,
};

pub use self::props::PropsUpdateWith;
pub use crate::{application::content::GlobalContent, dom::RcElementModel};

pub mod props;

#[macro_export]
macro_rules! ElModel {
    ()=>{
        $crate::ElModel!(Self)
    };

    (_)=>{
        $crate::ElModel!(impl $crate::Element)
    };

    ($Ty: ty) => {
        $crate::element::RcElementModel<
            $Ty,
            impl $crate::StyleGroup,
            impl $crate::ChildNodes
        >
    };
}

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

    fn render(this: &ElModel!(), content: RenderElement) -> Result<()>;
    fn on_pointer_event(this: &ElModel!(), ipe: &IncomingPointerEvent) -> bool;
}

pub trait ElementCreate<Pr>: Element + Sized {
    fn el_create(this: &ElModel!(), props: Pr) -> Self;
}

pub trait ElementPropsUpdate<Pr>: Element + Sized {
    fn el_update(&mut self, props: Pr);
}

pub trait AsChildren: ChildNodes {}
impl<T: ChildNodes> AsChildren for T {}
