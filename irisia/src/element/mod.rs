use std::{sync::Arc, time::Duration};

use crate::{dom::children::ChildrenNodes, primitive::Region, Result};

pub use self::{children_setter::ChildrenSetter, render_element::RenderElement};
pub use crate::dom::ElementHandle;

mod children_setter;
pub mod props;
mod render_element;

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

    fn layout<'a, Ch>(&mut self, draw_region: Region, children: Ch, setter: ChildrenSetter<'a>)
    where
        Ch: ChildrenNodes;

    /// Draw to the canvas
    fn render(
        &mut self,
        renderer: RenderElement,
        interval: Duration,
        draw_region: Region,
    ) -> Result<()>;
}

pub struct UpdateElement<'a, El, Pr, Sty> {
    pub props: Pr,
    pub styles: &'a Sty,
    pub handle: &'a Arc<ElementHandle<El>>,
}
