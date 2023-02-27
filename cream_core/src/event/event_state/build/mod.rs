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

pub struct EvlBuilder<Pl, El, L> {
    pub(super) handle: WeakHandle<ProxyLayerCache<Pl, El>>,
    pub(super) listeners: L,
}

impl<Pl, El> EvlBuilder<Pl, El, ()> {
    pub fn new(handle: &RcHandle<ProxyLayerCache<Pl, El>>) -> Self {
        EvlBuilder {
            handle: Rc::downgrade(&handle),
            listeners: (),
        }
    }
}

impl<T, El, L> EvlBuilder<T, El, L>
where
    L: EventResolve<T, El>,
{
    pub fn listen<E, F>(self, f: F) -> EvlBuilder<T, El, Chain<L, AddListener<E, F>>>
    where
        E: Event,
        F: Fn(&mut El, &E, &mut EventFlow),
    {
        EvlBuilder {
            handle: self.handle,
            listeners: Chain::new(self.listeners, AddListener::new(f)),
        }
    }

    pub fn chain<U>(self, other: U) -> EvlBuilder<T, El, Chain<L, U>>
    where
        U: EventResolve<T, El>,
    {
        EvlBuilder {
            handle: self.handle,
            listeners: Chain::new(self.listeners, other),
        }
    }
}

impl<T, El, L> Clone for EvlBuilder<T, El, L>
where
    L: Clone,
{
    fn clone(&self) -> Self {
        EvlBuilder {
            handle: self.handle.clone(),
            listeners: self.listeners.clone(),
        }
    }
}
