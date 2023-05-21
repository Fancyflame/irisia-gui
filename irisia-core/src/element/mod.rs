use crate::primitive::Region;
use crate::structure::{StructureBuilder, VisitIter};
use crate::{style::StyleContainer, Result};

use crate::structure::slot::Slot;

pub use element_handle::ElementHandle;
pub use render_content::RenderContent;
pub use runtime_init::RuntimeInit;
pub use state_update::{
    diff_watcher::{Gdw, Udw},
    StateUpdate,
};

pub mod element_handle;
pub mod render_content;
pub mod runtime_init;
pub mod state_update;

pub struct Frame<'a, 'prop, El, St, Ch>
where
    El: Element,
    Ch: StructureBuilder,
{
    pub props: El::Props<'prop>,
    pub styles: &'a St,
    pub drawing_region: Region,
    pub children: Slot<'a, Ch>,
    pub content: RenderContent<'a>,
    pub ri: &'a RuntimeInit<El>,
}

/// Element is a thing can draw itself on the given canvas,
/// according to its properties, styles and given drawing region.
/// This trait is close to the native rendering, if you are not a
/// component maker, please using exist element or using macros to
/// customize one.
pub trait Element: Sized + 'static {
    type Props<'a>: Default;
    type ChildProps<'a>;

    fn create(init: &RuntimeInit<Self>) -> Self;

    fn render<'a>(
        &mut self,
        frame: Frame<Self, impl StyleContainer, impl VisitIter<Self::ChildProps<'a>>>,
    ) -> Result<()>;

    fn compute_size(&self) -> (Option<u32>, Option<u32>) {
        (None, None)
    }
}

#[derive(Clone, Copy, Default)]
pub struct NoProps {}

pub trait PropsAsChild<'a, T>: Element {
    fn props_as_child(&'a self, props: &Self::Props<'_>, style: &impl StyleContainer) -> T;
}

impl<El: Element> PropsAsChild<'_, ()> for El {
    fn props_as_child(&self, _: &Self::Props<'_>, _: &impl StyleContainer) {}
}

impl<'a, El: Element> PropsAsChild<'a, &'a El> for El {
    fn props_as_child(&self, _: &Self::Props<'_>, _: &impl StyleContainer) -> &El {
        self
    }
}

pub struct NeverInitalized(());
