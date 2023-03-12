use crate::{
    event::event_state::{
        build::EventListenerBuilder, proxy::EvlProxyBuilder, wrap::WrappedEvents, EventResolve,
    },
    structure::{slot::Slot, Node},
    style::StyleContainer,
    Result,
};

use super::{Element, RenderContent};

pub trait ProxyLayer<El: Element>: Default + 'static {
    fn proxy<S, El2, Pl2, L, C>(
        &mut self,
        element: &mut El,
        props: El::Props<'_>,
        styles: &S,
        evl_proxy: EvlProxyBuilder<Pl2, El2, L>,
        evl_builder: EventListenerBuilder<Self, El, ()>,
        children: Slot<C>,
        content: RenderContent,
    ) -> Result<()>
    where
        S: StyleContainer,
        El2: Element,
        Pl2: ProxyLayer<El2>,
        L: EventResolve<Pl2, El2>,
        C: Node,
        El: Element<Children<C> = C>;
}

impl<El: Element> ProxyLayer<El> for () {
    fn proxy<S, El2, Pl2, L, C>(
        &mut self,
        element: &mut El,
        props: <El as Element>::Props<'_>,
        styles: &S,
        evl_proxy: EvlProxyBuilder<Pl2, El2, L>,
        evl_builder: EventListenerBuilder<Self, El, ()>,
        children: Slot<C>,
        content: RenderContent,
    ) -> Result<()>
    where
        S: StyleContainer,
        El2: Element,
        Pl2: ProxyLayer<El2>,
        L: EventResolve<Pl2, El2>,
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
