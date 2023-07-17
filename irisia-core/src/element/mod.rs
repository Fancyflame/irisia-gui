use crate::structure::StructureBuilder;
use crate::Result;

pub use init_content::{element_handle::ElementHandle, InitContent};
pub use render_content::RenderContent;
pub use state::StateUpdate;

use self::state::ElProps;

pub mod init_content;
pub mod render_content;
pub mod state;

/// Element is a thing can draw itself on the given canvas,
/// according to its properties, styles and given drawing region.
/// This trait is close to the native rendering, if you are not a
/// component maker, please using exist element or using macros tos
/// customize one.
pub trait Element<Sb>
where
    Self: Sized + 'static,
    Sb: StructureBuilder,
{
    type Props: ElProps;

    fn create(init: &InitContent<Self>) -> Self;
    fn render(&mut self, frame: Frame<Self, Sb>) -> Result<()>;
    fn compute_size(&self) -> (Option<u32>, Option<u32>) {
        (None, None)
    }
}
pub struct Frame<'a, El, Sb>
where
    El: Element<Sb>,
    Sb: StructureBuilder,
{
    pub props: &'a El::Props,
    pub children: Sb,
    pub content: RenderContent<'a>,
}

#[derive(Clone, Copy, Default)]
pub struct NoProps {}

pub trait AsChildProps<'a, T> {
    fn as_child_props(&'a self) -> T;
}
