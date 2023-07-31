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
pub use props::UpdateFrom;

pub mod init_content;
pub mod props;

pub(crate) type ChildrenCache<El, Pr, Str> =
    <<<El as ElementMutate<Pr, Str>>::Children as Structure>::Activated as ActivatedStructure>::Cache;

pub(crate) type SelfCache<Str> = <<Str as Structure>::Activated as ActivatedStructure>::Cache;

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
    fn render(&mut self, canvas: &mut Canvas, draw_region: Region) -> Result<()>;
}

pub trait ElementMutate<Pr, Str>
where
    Str: Structure,
    Self: Sized,
{
    type Children: Structure;

    fn compute_size(props: &Pr, styles: &impl StyleContainer, children: &Str) -> ComputeSize {
        let _ = (props, styles, children);
        Default::default()
    }

    fn create<Sty>(
        init: &InitContent<Self>,
        args: UpdateArguments<Pr, Sty, Str, Self::Children>,
    ) -> Self
    where
        Sty: StyleContainer;

    fn update<Sty>(&mut self, args: UpdateArguments<Pr, Sty, Str, Self::Children>)
    where
        Sty: StyleContainer;
}

pub struct UpdateArguments<'a, Pr, Sty, Str, Sb2>
where
    Sb2: Structure,
{
    pub props: Pr,
    pub styles: Sty,
    pub children: Str,
    pub equality_matters: bool,
    pub updater: UpdateElementContent<'a, Sb2>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ComputeSize(Option<Pixel>, Option<Pixel>);
