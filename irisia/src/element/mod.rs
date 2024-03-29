use crate::{dom::RenderMultiple, primitive::Region, Result};

pub use self::{props::PropsUpdateWith, render_element::RenderElement};
pub use crate::{
    application::content::GlobalContent,
    dom::{one_child, pub_handle::LayoutElements, RcElementModel},
};

pub mod props;
mod render_element;

#[macro_export]
macro_rules! ElModel {
    () => {
        $crate::ElModel!(Self)
    };
    ($El: ty) => {
        $crate::element::RcElementModel<
            $El,
            impl $crate::style::StyleContainer + 'static,
            impl $crate::element::AsChildren + 'static
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

    fn render(&mut self, this: &ElModel!(), mut content: RenderElement) -> Result<()> {
        let _ = this;
        content.render_children()
    }

    fn set_children(&self, this: &ElModel!()) {
        this.set_children(()).layout(|()| unreachable!()).unwrap();
    }

    fn draw_region_changed(&mut self, this: &ElModel!(), draw_region: Region) {
        if let Some(lc) = this.layout_children() {
            lc.layout_once(draw_region)
                .expect("child elements are more than 1")
        }
    }
}

pub trait ElementUpdate<Pr>: Element + Sized {
    fn el_create(this: &ElModel!(), props: Pr) -> Self;
    fn el_update(&mut self, this: &ElModel!(), props: Pr, equality_matters: bool) -> bool;
}

pub trait AsChildren: RenderMultiple {}
impl<T: RenderMultiple> AsChildren for T {}
