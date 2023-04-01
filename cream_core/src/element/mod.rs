use std::sync::Arc;

use crate::event::event_dispatcher::RecvOnly;
use crate::event::{EventDispatcher, EventEmitter, EventReceive};
use crate::primary::Region;
use crate::structure::{StructureBuilder, VisitIter};
use crate::{style::StyleContainer, Result};
use crate::{CacheBox, Event};

use crate::structure::slot::Slot;

use cream_backend::window_handle::close_handle::CloseHandle;
pub use render_content::RenderContent;
use tokio::sync::Mutex;

pub mod render_content;
pub mod render_fn_macro;

/// Element is a thing can draw itself on the given canvas,
/// according to its properties, styles and given drawing region.
/// This trait is close to the native rendering, if you are not a
/// component maker, please using exist element or using macros to
/// custom one.
pub trait Element: 'static {
    type Props<'a>: Default;
    type ChildProps<'a>;

    fn create() -> Self;

    fn render<'r>(
        &mut self,
        props: Self::Props<'_>,
        styles: &impl StyleContainer,
        drawing_region: Region,
        cache_box_for_children: &mut CacheBox,
        event_dispatcher: &EventDispatcher,
        children: Slot<impl StructureBuilder + VisitIter<Self::ChildProps<'r>>>,
        content: RenderContent,
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
    fn props_as_child(&self, _: &Self::Props<'_>, _: &impl StyleContainer) -> () {
        ()
    }
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
    pub output_event_emitter: EventEmitter,
    pub event_dispatcher: EventDispatcher,
    pub window_event_dispatcher: RecvOnly,
    pub close_handle: CloseHandle,
}

impl<T: ?Sized> RuntimeInit<T> {
    pub fn recv<E, K>(&self) -> EventReceive<E, K>
    where
        E: Event,
        K: Clone + Send + Unpin + 'static,
    {
        self.event_dispatcher.recv()
    }
}
