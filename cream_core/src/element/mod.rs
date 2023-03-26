use std::sync::Arc;

use crate::event::{EventChanGetter, EventChanSetter, EventEmitter, EventReceiver};
use crate::primary::Region;
use crate::structure::StructureBuilder;
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

#[derive(Clone, Copy, Default)]
pub struct NoProps {}

pub trait Element: Send + 'static {
    type Props<'a>: Default;
    fn create() -> Self;

    fn render(
        &mut self,
        props: Self::Props<'_>,
        styles: &impl StyleContainer,
        drawing_region: Region,
        cache_box_for_children: &mut CacheBox,
        chan_setter: &EventChanSetter,
        children: Slot<impl StructureBuilder>,
        content: RenderContent,
    ) -> Result<()>;

    fn compute_size(&self) -> (Option<u32>, Option<u32>) {
        (None, None)
    }

    fn start_runtime(init: RuntimeInit<Self>) {
        let _ = init;
    }
}

pub struct RuntimeInit<T: ?Sized> {
    pub(crate) _prevent_new: (),
    pub app: Arc<Mutex<T>>,
    pub event_emitter: EventEmitter,
    pub channels: EventChanGetter,
    pub close_handle: CloseHandle,
}

impl<T> RuntimeInit<T> {
    pub async fn emit<E>(&self, event: &E)
    where
        E: Event + Clone,
    {
        self.event_emitter.emit(event).await
    }

    pub async fn get_receiver(&self, name: &'static str) -> EventReceiver {
        self.channels.get_receiver(name).await
    }
}
