use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
};

use smallvec::SmallVec;

use crate::map_rc::{MapRc, MapWeak};

use super::{callback::ClosureCall, Event, EventFlow};

const CALLBACK_LIST_LEN: usize = 6;
type EventListener<A> = MapWeak<dyn ClosureCall<A>>;
pub(crate) type EventListenerList<A> = RefCell<SmallVec<[EventListener<A>; CALLBACK_LIST_LEN]>>;

pub struct EventTarget {
    // dyn Any -> EventListenerList<E::Arg>
    event_listeners: HashMap<TypeId, MapRc<dyn Any>>,
}

impl EventTarget {
    pub(crate) fn new() -> Self {
        EventTarget {
            event_listeners: HashMap::new(),
        }
    }

    pub(crate) fn get<E: Event>(&self, ev: E) -> Option<MapRc<EventListenerList<E::Arg>>> {
        self.event_listeners.get(&ev.type_id()).map(|ell| {
            MapRc::map(ell, |any| {
                any.downcast_ref::<EventListenerList<E::Arg>>()
                    .expect("unreachable")
            })
        })
    }

    pub(crate) fn get_or_create<E: Event>(&mut self, ev: E) -> MapRc<EventListenerList<E::Arg>> {
        match self.get(ev) {
            Some(s) => s,
            None => {
                let map_rc: MapRc<EventListenerList<E::Arg>> =
                    MapRc::new(RefCell::new(SmallVec::new()));

                self.event_listeners
                    .insert(ev.type_id(), MapRc::map_to_any(&map_rc));
                map_rc
            }
        }
    }

    pub(crate) fn on<E>(&mut self, ev: E, el: EventListener<E::Arg>)
    where
        E: Event,
    {
        (*self.get_or_create(ev)).borrow_mut().push(el);
    }

    pub fn emit<E>(&mut self, ev: E, args: &E::Arg, event_flow: &mut EventFlow)
    where
        E: Event,
    {
        if let Some(ell) = self.get(ev) {
            (*ell).borrow_mut().retain(|callback| {
                let callback = match callback.upgrade() {
                    Some(cb) => cb,
                    None => return false,
                };

                callback.call(args, event_flow);
                true
            });
        }
    }
}
