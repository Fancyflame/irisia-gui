use std::sync::Arc;

use crate::event::{EventChanGetter, EventChanSetter, EventEmitter};
use crate::CacheBox;
use crate::{style::StyleContainer, Result};

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

    fn start_runtime(
        slf: Arc<Mutex<Self>>,
        event_emitter: EventEmitter,
        chan_getter: EventChanGetter,
        close_handle: CloseHandle,
    ) {
        let _ = (slf, event_emitter, chan_getter, close_handle);
    }
}
