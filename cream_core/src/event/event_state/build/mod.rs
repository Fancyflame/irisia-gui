use std::rc::Rc;

use crate::{
    element::{RcHandle, WeakHandle},
    event::{Event, EventFlow},
    structure::add_child::pl_cache::ProxyLayerCache,
};

use self::{add_listener::AddListener, chain::Chain};

use super::EventResolve;

pub mod add_listener;
pub mod chain;
pub mod empty;

pub struct EventListenerBuilder<Pl, El, L> {
    pub(super) handle: WeakHandle<ProxyLayerCache<Pl, El>>,
    pub(super) listeners: L,
}

impl<Pl, El> EventListenerBuilder<Pl, El, ()> {
    pub fn new(handle: &RcHandle<ProxyLayerCache<Pl, El>>) -> Self {
        EventListenerBuilder {
            handle: Rc::downgrade(&handle),
            listeners: (),
        }
    }
}

impl<T, El, L> EventListenerBuilder<T, El, L>
where
    L: EventResolve<T, El>,
{
    pub fn listen<E, F>(self, f: F) -> EventListenerBuilder<T, El, Chain<L, AddListener<E, F>>>
    where
        E: Event,
        F: Fn(&mut El, &E, &mut EventFlow),
    {
        EventListenerBuilder {
            handle: self.handle,
            listeners: Chain::new(self.listeners, AddListener::new(f)),
        }
    }

    pub fn chain<U>(self, other: U) -> EventListenerBuilder<T, El, Chain<L, U>>
    where
        U: EventResolve<T, El>,
    {
        EventListenerBuilder {
            handle: self.handle,
            listeners: Chain::new(self.listeners, other),
        }
    }
}

impl<T, El, L> Clone for EventListenerBuilder<T, El, L>
where
    L: Clone,
{
    fn clone(&self) -> Self {
        EventListenerBuilder {
            handle: self.handle.clone(),
            listeners: self.listeners.clone(),
        }
    }
}
