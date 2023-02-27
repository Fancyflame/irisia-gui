use std::any::Any;

use crate::{event::event_state::EventResolve, structure::add_child::pl_cache::ProxyLayerCache};

pub struct Chain<A, B>(A, B);

impl<A, B> Chain<A, B> {
    pub(super) fn new(before: A, after: B) -> Self {
        Chain(before, after)
    }
}

impl<Pl, El, A, B> EventResolve<Pl, El> for Chain<A, B>
where
    A: EventResolve<Pl, El>,
    B: EventResolve<Pl, El>,
{
    const IS_EMPTY: bool = A::IS_EMPTY && B::IS_EMPTY;

    fn is_related(tid: std::any::TypeId) -> bool {
        A::is_related(tid) || B::is_related(tid)
    }

    fn callback(
        &mut self,
        src: &mut ProxyLayerCache<Pl, El>,
        arg: &dyn Any,
        flow: &mut crate::event::EventFlow,
    ) {
        self.0.callback(src, arg, flow);
        self.1.callback(src, arg, flow);
    }
}
