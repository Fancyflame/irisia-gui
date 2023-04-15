use std::sync::Arc;

use crate::event::{EventDispatcher, EventReceive};
use crate::primary::Region;
use crate::structure::{StructureBuilder, VisitIter};
use crate::Event;
use crate::{style::StyleContainer, Result};

use crate::structure::slot::Slot;

use irisia_backend::window_handle::close_handle::CloseHandle;
pub use render_content::RenderContent;
use tokio::sync::Mutex;

pub mod render_content;
pub mod render_fn_macro;

pub struct Frame<'a, 'prop, El, St, Ch>
where
    El: Element,
    Ch: StructureBuilder,
{
    pub props: El::Props<'prop>,
    pub styles: &'a St,
    pub drawing_region: Region,
    pub event_dispatcher: &'a EventDispatcher,
    pub children: Slot<'a, Ch>,
    pub content: RenderContent<'a>,
}

/// Element is a thing can draw itself on the given canvas,
/// according to its properties, styles and given drawing region.
/// This trait is close to the native rendering, if you are not a
/// component maker, please using exist element or using macros to
/// custom one.
pub trait Element: Default + Sized + 'static {
    type Props<'a>: Default;
    type ChildProps<'a>;

    fn render<'a>(
        &mut self,
        frame: Frame<Self, impl StyleContainer, impl VisitIter<Self::ChildProps<'a>>>,
    ) -> Result<()>;

    fn compute_size(&self) -> (Option<u32>, Option<u32>) {
        (None, None)
    }

    fn start_runtime(init: RuntimeInit<Self>) {
        let _ = init;
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

pub struct NeverInitalized {
    _never_initialized: (),
}

pub struct RuntimeInit<T: ?Sized> {
    pub(crate) _prevent_new: (),
    pub app: Arc<Mutex<T>>,
    pub event_dispatcher: EventDispatcher,
    pub window_event_dispatcher: EventDispatcher,
    pub close_handle: CloseHandle,
}

impl<T: ?Sized> RuntimeInit<T> {
    pub fn recv<E: Event>(&self) -> EventReceive<E> {
        self.event_dispatcher.recv()
    }

    pub async fn recv_sys<E: Event>(&self) -> E {
        self.event_dispatcher.recv_sys().await
    }
}
