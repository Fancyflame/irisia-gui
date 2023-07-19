use crate::primitive::Region;
use crate::structure::StructureBuilder;
use crate::style::Pixel;
use crate::style::StyleContainer;
use crate::Result;

pub use init_content::{element_handle::ElementHandle, InitContent};
pub use props::StateUpdate;
pub use render_content::RenderContent;

pub mod init_content;
pub mod props;
pub mod render_content;

/// Element is a thing can draw itself on the given canvas,
/// according to its properties, styles and given drawing region.
/// This trait is close to the native rendering, if you are not a
/// component maker, please using exist element or using macros tos
/// customize one.
pub trait Element
where
    Self: Sized + 'static,
{
    type BlankProps: Default;

    /// Draw to the canvas
    fn render(&mut self, content: RenderContent) -> Result<()>;
}

pub trait ElementMutate<Pr, Sb>
where
    Sb: StructureBuilder,
    Self: Sized,
{
    fn compute_size(props: &Pr, styles: &impl StyleContainer, children: &Sb) -> ComputeSize {
        let _ = (props, styles, children);
        Default::default()
    }

    fn create<Sty>(init: &InitContent<Self>, args: UpdateArguments<Pr, Sty, Sb>) -> Self
    where
        Sty: StyleContainer;

    fn update<Sty>(&mut self, args: UpdateArguments<Pr, Sty, Sb>) -> bool
    where
        Sty: StyleContainer;
}

pub struct UpdateArguments<Pr, Sty, Sb> {
    pub props: Pr,
    pub styles: Sty,
    pub children: Sb,
    pub draw_region: Region,
    pub equality_matters: bool,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ComputeSize(Option<Pixel>, Option<Pixel>);
