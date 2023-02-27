use crate::{
    event::event_state::{
        build::EvlBuilder, proxy::EvlProxyBuilder, wrap::WrappedEvents, EventResolve,
    },
    structure::{slot::Slot, Node},
    style::StyleContainer,
    Result,
};

use super::{Element, RenderContent};

pub trait ProxyLayer<El: Element>: Default + 'static {
    fn proxy<S, L, C>(
        &mut self,
        element: &mut El,
        props: El::Props<'_>,
        styles: &S,
        evl_proxy: EvlProxyBuilder<Self, El, L>,
        evl_builder: EvlBuilder<Self, El, ()>,
        children: Slot<C>,
        content: RenderContent,
    ) -> Result<()>
    where
        S: StyleContainer,
        L: EventResolve<Self, El>,
        C: Node,
        El: Element<Children<C> = C>;
}

impl<El: Element> ProxyLayer<El> for () {
    fn proxy<S, L, C>(
        &mut self,
        element: &mut El,
        props: <El as Element>::Props<'_>,
        styles: &S,
        evl_proxy: EvlProxyBuilder<Self, El, L>,
        evl_builder: EvlBuilder<Self, El, ()>,
        children: Slot<C>,
        content: RenderContent,
    ) -> Result<()>
    where
        S: StyleContainer,
        L: EventResolve<Self, El>,
        C: Node,
        El: Element<Children<C> = C>,
    {
        element.render(
            props,
            styles,
            WrappedEvents::from_proxy(evl_proxy),
            evl_builder,
            children,
            content,
        )
    }
}
