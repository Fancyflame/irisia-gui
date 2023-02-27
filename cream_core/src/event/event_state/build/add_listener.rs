use std::{
    any::{Any, TypeId},
    marker::PhantomData,
};

use crate::{
    event::{event_state::EventResolve, Event, EventFlow},
    structure::add_child::pl_cache::ProxyLayerCache,
};

pub struct AddListener<E, F>
where
    E: Event,
{
    _phantom: PhantomData<E>,
    callback: F,
}

impl<E, F> AddListener<E, F>
where
    E: Event,
{
    pub(super) fn new(f: F) -> Self {
        AddListener {
            _phantom: PhantomData,
            callback: f,
        }
    }
}

impl<Pl, El, E, F> EventResolve<Pl, El> for AddListener<E, F>
where
    E: Event,
    F: FnMut(&mut El, &E, &mut EventFlow) + 'static,
{
    const IS_EMPTY: bool = false;

    fn is_related(tid: std::any::TypeId) -> bool {
        tid == TypeId::of::<E>()
    }

    fn callback(&mut self, src: &mut ProxyLayerCache<Pl, El>, arg: &dyn Any, flow: &mut EventFlow) {
        if let Some(arg) = arg.downcast_ref() {
            (self.callback)(&mut src.elem, arg, flow);
        }
    }
}
