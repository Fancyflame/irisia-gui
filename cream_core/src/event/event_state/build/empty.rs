use std::any::{Any, TypeId};

use crate::{
    event::{event_state::EventResolve, EventFlow},
    structure::add_child::pl_cache::ProxyLayerCache,
};

impl<Pl, El> EventResolve<Pl, El> for () {
    const IS_EMPTY: bool = true;

    fn is_related(_: TypeId) -> bool {
        false
    }

    fn callback(
        &mut self,
        _src: &mut ProxyLayerCache<Pl, El>,
        _arg: &dyn Any,
        _flow: &mut EventFlow,
    ) {
    }
}
