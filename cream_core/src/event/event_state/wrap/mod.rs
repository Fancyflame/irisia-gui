use std::any::{Any, TypeId};

use crate::{
    element::{proxy_layer::ProxyLayer, Element, RcHandle},
    event::{Event, EventFlow},
};

use super::{proxy::EvlProxyBuilder, EventResolve};

mod inner;

trait CompleteEventResolve {
    fn is_related(&self, tid: TypeId) -> bool;
    fn callback(&mut self, event: &dyn Any, flow: &mut EventFlow);
}

pub struct WrappedEvents(Option<RcHandle<dyn CompleteEventResolve>>);

impl WrappedEvents {
    pub(crate) fn from_proxy<Pl, El, L>(proxy: EvlProxyBuilder<Pl, El, L>) -> Self
    where
        Pl: ProxyLayer<El>,
        El: Element,
        L: EventResolve<Pl, El>,
    {
        WrappedEvents(inner::wrap_proxy(proxy))
    }

    pub(crate) fn new_empty() -> Self {
        WrappedEvents(None)
    }

    pub fn emit<E: Event>(&self, event: &E, flow: &mut EventFlow) {
        if let Some(inner) = &self.0 {
            inner.borrow_mut().callback(event, flow);
        }
    }

    pub fn is_related<E: Event>(&self) -> bool {
        if let Some(inner) = &self.0 {
            inner.borrow().is_related(TypeId::of::<E>())
        } else {
            false
        }
    }
}

impl Clone for WrappedEvents {
    fn clone(&self) -> Self {
        WrappedEvents(self.0.clone())
    }
}
