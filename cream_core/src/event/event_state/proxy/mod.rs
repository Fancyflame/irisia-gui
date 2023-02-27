use crate::{
    element::{proxy_layer::ProxyLayer, Element},
    event::{Event, EventFlow},
};

use self::{
    add_listener2::AddListener2,
    proxy::{OriginalEvent, Proxy},
};

use super::{
    build::{chain::Chain, EvlBuilder},
    EventResolve,
};

pub mod add_listener2;
pub mod proxy;

pub struct EvlProxyBuilder<Pl, El, L>(pub(super) EvlBuilder<Pl, El, L>);

impl<Pl, El, L> EvlProxyBuilder<Pl, El, L>
where
    Pl: ProxyLayer<El>,
    El: Element,
    L: EventResolve<Pl, El>,
{
    pub(crate) fn from_builder(builder: EvlBuilder<Pl, El, L>) -> Self {
        EvlProxyBuilder(builder)
    }

    pub fn listen<E, F>(self, f: F) -> EvlProxyBuilder<Pl, El, Chain<L, AddListener2<E, F>>>
    where
        E: Event,
        F: Fn(&mut Pl, &E, &mut EventFlow) + 'static,
    {
        EvlProxyBuilder(self.0.chain(AddListener2::new(f)))
    }

    pub fn chain<U>(self, other: U) -> EvlProxyBuilder<Pl, El, Chain<L, U>>
    where
        U: EventResolve<Pl, El>,
    {
        EvlProxyBuilder(self.0.chain(other))
    }

    pub fn proxy<E, F>(self, f: F) -> EvlProxyBuilder<Pl, El, Proxy<E, L, F>>
    where
        E: Event,
        F: FnMut(OriginalEvent<Pl, El, E, L>),
    {
        EvlProxyBuilder(EvlBuilder {
            handle: self.0.handle,
            listeners: Proxy::new(self.0.listeners, f),
        })
    }
}
