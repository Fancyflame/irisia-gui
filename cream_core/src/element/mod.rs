use std::rc::Weak;
use std::{cell::RefCell, rc::Rc};

use crate::event::event_state::build::EventListenerBuilder;
use crate::event::event_state::wrap::WrappedEvents;
use crate::{style::StyleContainer, Result};

use crate::structure::{slot::Slot, Node};

pub use render_content::RenderContent;

use self::proxy_layer::ProxyLayer;

pub mod proxy_layer;
pub mod render_content;

/// Element is a thing can draw itself on the given canvas,
/// according to its properties, styles and given drawing region.
/// This trait is close to the native rendering, if you are not a
/// component maker, please using exist element or using macros to
/// custom one.

#[derive(Clone, Copy, Default)]
pub struct NoProps {}

pub type RcHandle<T> = Rc<RefCell<T>>;
pub type WeakHandle<T> = Weak<RefCell<T>>;

pub trait Element: Default + 'static {
    type Props<'a>: Default;
    type Children<Ch>: Node
    where
        Ch: Node;

    fn render<S, Pl, C>(
        &mut self,
        props: Self::Props<'_>,
        styles: &S,
        event_listeners: WrappedEvents,
        event_listener_builder: EventListenerBuilder<Pl, Self, ()>,
        children: Slot<C>,
        content: RenderContent,
    ) -> Result<()>
    where
        S: StyleContainer,
        Pl: ProxyLayer<Self>,
        C: Node,
        Self: Element<Children<C> = C>;
}
