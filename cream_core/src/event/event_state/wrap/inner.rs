use std::{any::TypeId, cell::RefCell, rc::Rc};

use crate::{
    element::{proxy_layer::ProxyLayer, Element, RcHandle, WeakHandle},
    event::event_state::{proxy::EvlProxyBuilder, EventResolve},
    structure::add_child::pl_cache::ProxyLayerCache,
};

use super::CompleteEventResolve;

struct EvListenerInner<Pl, El, L> {
    handle: WeakHandle<ProxyLayerCache<Pl, El>>,
    listeners: L,
}

impl<Pl, El, L> CompleteEventResolve for EvListenerInner<Pl, El, L>
where
    L: EventResolve<Pl, El>,
{
    fn is_related(&self, tid: TypeId) -> bool {
        L::is_related(tid)
    }

    fn callback(&mut self, event: &dyn std::any::Any, flow: &mut crate::event::EventFlow) {
        if let Some(rc) = self.handle.upgrade() {
            self.listeners.callback(&mut rc.borrow_mut(), event, flow)
        }
    }
}

pub(super) fn wrap_proxy<Pl, El, L>(
    proxy: EvlProxyBuilder<Pl, El, L>,
) -> Option<RcHandle<dyn CompleteEventResolve>>
where
    Pl: ProxyLayer<El>,
    El: Element,
    L: EventResolve<Pl, El> + 'static,
{
    if L::IS_EMPTY {
        return None;
    }

    let inner = EvListenerInner {
        handle: proxy.0.handle,
        listeners: proxy.0.listeners,
    };

    Some(Rc::new(RefCell::new(inner)) as _)
}
