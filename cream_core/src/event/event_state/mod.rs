use std::any::{Any, TypeId};

use crate::{event::EventFlow, structure::add_child::pl_cache::ProxyLayerCache};

pub mod build;
pub mod proxy;
pub mod wrap;

pub trait EventResolve<Pl, El>: 'static {
    const IS_EMPTY: bool;
    fn is_related(tid: TypeId) -> bool;
    fn callback(
        &mut self,
        src: &mut ProxyLayerCache<Pl, El>,
        event: &dyn Any,
        flow: &mut EventFlow,
    );
}
