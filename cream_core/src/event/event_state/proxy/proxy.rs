use std::marker::PhantomData;

use crate::{
    event::{event_state::EventResolve, Event, EventFlow},
    structure::add_child::pl_cache::ProxyLayerCache,
};

pub struct Proxy<E, L, F> {
    _phantom: PhantomData<E>,
    listeners: L,
    func: F,
}

impl<E, L, F> Proxy<E, L, F> {
    pub(super) fn new(listeners: L, func: F) -> Self {
        Proxy {
            _phantom: PhantomData,
            listeners,
            func,
        }
    }
}

impl<Pl, El, E, L, F> EventResolve<Pl, El> for Proxy<E, L, F>
where
    E: Event,
    L: EventResolve<Pl, El>,
    F: FnMut(OriginalEvent<'_, Pl, El, E, L>) + 'static,
{
    const IS_EMPTY: bool = L::IS_EMPTY;
    fn is_related(tid: std::any::TypeId) -> bool {
        L::is_related(tid)
    }

    fn callback(
        &mut self,
        src: &mut ProxyLayerCache<Pl, El>,
        event: &dyn std::any::Any,
        flow: &mut crate::event::EventFlow,
    ) {
        match event.downcast_ref::<E>() {
            None => self.listeners.callback(src, event, flow),
            Some(event) => {
                let mut delay_emit_sig = true;
                let origin = OriginalEvent {
                    delay_emit_sig: &mut delay_emit_sig,
                    src,
                    event,
                    flow,
                    event_listeners: &mut self.listeners,
                };
                (self.func)(origin);

                if delay_emit_sig {
                    self.listeners.callback(src, event, flow);
                }
            }
        }
    }
}

pub struct OriginalEvent<'a, Pl, El, E, L> {
    delay_emit_sig: &'a mut bool,
    src: &'a mut ProxyLayerCache<Pl, El>,
    event: &'a E,
    flow: &'a mut EventFlow,
    event_listeners: &'a mut L,
}

impl<'a, Pl, El, E, L> OriginalEvent<'a, Pl, El, E, L>
where
    E: Event,
    L: EventResolve<Pl, El>,
{
    pub fn before(self) -> EventExpander<'a, Pl, El, L> {
        *self.delay_emit_sig = false;
        self.event_listeners
            .callback(self.src, self.event, self.flow);
        self.return_expander()
    }

    // default behavior
    pub fn after(self) -> EventExpander<'a, Pl, El, L> {
        self.return_expander()
    }

    pub fn block(self) -> EventExpander<'a, Pl, El, L> {
        *self.delay_emit_sig = false;
        self.return_expander()
    }

    fn return_expander(self) -> EventExpander<'a, Pl, El, L> {
        EventExpander {
            src: self.src,
            event_listeners: self.event_listeners,
        }
    }
}

pub struct EventExpander<'a, Pl, El, L> {
    src: &'a mut ProxyLayerCache<Pl, El>,
    event_listeners: &'a mut L,
}

impl<Pl, El, L> EventExpander<'_, Pl, El, L>
where
    L: EventResolve<Pl, El>,
{
    pub fn new_event<E: Event>(&mut self, event: &E) {
        let mut flow = EventFlow {
            bubble: false,
            is_current: true,
        };

        self.event_listeners
            .callback(self.src, event as _, &mut flow);
    }
}
