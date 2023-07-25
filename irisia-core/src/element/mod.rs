use crate::{
    primitive::{Pixel, Region},
    structure::{
        activate::ActivatedStructure, activate::Structure,
        node::add_child::update_el::UpdateElementContent,
    },
    style::StyleContainer,
    Result,
};

pub use init_content::{event_handle::EventHandle, InitContent};
use irisia_backend::skia_safe::Canvas;
pub use props::StateUpdate;

pub mod init_content;
pub mod props;

pub(crate) type ChildrenCache<El, Pr, Sb> =
    <<<El as ElementMutate<Pr, Sb>>::Children as Structure>::Activated as ActivatedStructure>::Cache;

pub(crate) type SelfCache<Sb> = <<Sb as Structure>::Activated as ActivatedStructure>::Cache;

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
    fn render(&mut self, canvas: &mut Canvas) -> Result<()>;
}

pub trait ElementMutate<Pr, Sb>
where
    Sb: Structure,
    Self: Sized,
{
    type Children: Structure;

    fn compute_size(props: &Pr, styles: &impl StyleContainer, children: &Sb) -> ComputeSize {
        let _ = (props, styles, children);
        Default::default()
    }

    fn create<Sty>(
        init: &InitContent<Self>,
        args: UpdateArguments<Pr, Sty, Sb, Self::Children>,
    ) -> Self
    where
        Sty: StyleContainer;

    fn update<Sty>(&mut self, args: UpdateArguments<Pr, Sty, Sb, Self::Children>)
    where
        Sty: StyleContainer;
}

pub struct UpdateArguments<'a, Pr, Sty, Sb, Sb2>
where
    Sb2: Structure,
{
    pub props: Pr,
    pub styles: Sty,
    pub children: Sb,
    pub draw_region: Region,
    pub equality_matters: bool,
    pub updater: UpdateElementContent<'a, Sb2>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ComputeSize(Option<Pixel>, Option<Pixel>);
