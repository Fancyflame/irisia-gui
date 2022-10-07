use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use smallvec::SmallVec;

use crate::map_rc::{MapRc, MapWeak};

use super::{callback::ClosureCall, Event, EventFlow};

const CALLBACK_LIST_LEN: usize = 6;
type EventListener<A> = MapWeak<dyn ClosureCall<A>>;
pub type EventListenerList<A> = SmallVec<[EventListener<A>; CALLBACK_LIST_LEN]>;

pub struct EventTarget {
    // dyn Any -> EventListenerList<E::Arg>
    event_listeners: HashMap<TypeId, MapRc<dyn Any>>,
}

impl EventTarget {
    pub fn get<E: Event>(&self, ev: E) -> &[EventListener<E::Arg>] {
        match self.event_listeners.get(&ev.type_id()) {
            Some(map_rc) => map_rc
                .downcast_ref::<EventListenerList<E::Arg>>()
                .expect("unreachable"),
            None => &[],
        }
    }

    fn get_mut<E: Event>(&mut self, ev: E) -> &mut EventListenerList<E::Arg> {
        let type_id = ev.type_id();
        match self.event_listeners.get_mut(&type_id) {
            Some(vec) => vec.downcast_mut().unwrap(),
            None => {
                let map_rc: MapRc<dyn Any> =
                    MapRc::new(EventListenerList::<E::Arg>::new()).map(|ell| ell as _);

                self.event_listeners.insert(type_id, map_rc);

                self.event_listeners
                    .get_mut(&type_id)
                    .unwrap()
                    .downcast_mut()
                    .unwrap()
            }
        }
    }

    pub fn on<E>(&mut self, ev: E, el: EventListener<E::Arg>)
    where
        E: Event,
    {
        self.get_mut(ev).push(el);
    }

    pub fn call<E>(&mut self, ev: E, args: &E::Arg, event_flow: &mut EventFlow)
    where
        E: Event,
    {
        self.get_mut(ev).retain(|callback| {
            let callback = match callback.upgrade() {
                Some(cb) => cb,
                None => return false,
            };

            callback.call(args, event_flow);
            true
        });
    }
}
