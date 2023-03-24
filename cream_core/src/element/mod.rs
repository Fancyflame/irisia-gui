use std::sync::Arc;

use crate::event::{EventChanGetter, EventChanSetter, EventEmitter, EventReceiver};
use crate::{style::StyleContainer, Result};
use crate::{CacheBox, Event};

use crate::structure::{slot::Slot, Node};

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
    type Children<Ch>: Node
    where
        Ch: Node;

    fn create() -> Self;

    fn render<S, C>(
        &mut self,
        props: Self::Props<'_>,
        styles: &S,
        chan_setter: &EventChanSetter,
        cache_box: &mut CacheBox,
        children: Slot<C>,
        content: RenderContent,
    ) -> Result<()>
    where
        S: StyleContainer,
        C: Node,
        Self: Element<Children<C> = C>;

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
