use crate::{dom::RenderMultiple, primitive::Region, style::StyleContainer, Result};

pub use self::render_element::RenderElement;
pub use crate::{application::content::GlobalContent, dom::RcElementModel};

pub mod props;
mod render_element;

macro_rules! ElModel {
    () => {
        RcElementModel<Self, impl StyleContainer, impl RenderMultiple>
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
    fn render(&mut self, content: RenderElement) -> Result<()>;

    fn draw_region_changed(&mut self, model: &ElModel!(), draw_region: Region);
}

pub trait ElementUpdate<Pr>: Sized {
    fn el_create(model: &ElModel!(), props: Pr) -> Self;
    fn el_update(&mut self, model: &ElModel!(), props: Pr, equality_matters: bool) -> bool;
}

pub struct UpdateElement<'a, Pr, El, Sty, Sc> {
    pub props: Pr,
    pub this: &'a RcElementModel<El, Sty, Sc>,
}
