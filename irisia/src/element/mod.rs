use crate::{dom::RenderMultiple, primitive::Region, Result};

pub use self::render_element::RenderElement;
pub use crate::{application::content::GlobalContent, dom::RcElementModel};

pub mod props;
mod render_element;

#[macro_export]
macro_rules! EModel {
    () => {
        &RcElementModel<
            Self,
            impl $crate::style::StyleContainer,
            impl $crate::element::AsChildren
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

    /// Draw to the canvas
    fn render(&mut self, this: EModel!(), content: RenderElement) -> Result<()>;

    fn draw_region_changed(&mut self, this: EModel!(), draw_region: Region);
}

pub trait ElementUpdate<Pr>: Element + Sized {
    fn el_create(this: EModel!(), props: Pr) -> Self;
    fn el_update(&mut self, this: EModel!(), props: Pr, equality_matters: bool) -> bool;
}

pub trait AsChildren: RenderMultiple {}
impl<T: RenderMultiple> AsChildren for T {}
