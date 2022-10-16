use std::{
    any::TypeId,
    collections::{hash_map, HashMap},
    rc::Rc,
};

use super::{
    callback::{AnonymousCallback, IntoCallback},
    Event, EventFlow,
};

pub type EventsIter<'a> = hash_map::Keys<'a, TypeId, (usize, usize)>;

struct EventListenerStorage {
    callback: AnonymousCallback,
    next: Option<usize>,
}

pub struct EventTarget {
    listeners: Vec<EventListenerStorage>,
    listeners_header_tail: HashMap<TypeId, (usize, usize)>,
}

impl EventTarget {
    pub(crate) fn new() -> Self {
        EventTarget {
            listeners: Vec::new(),
            listeners_header_tail: HashMap::new(),
        }
    }

    pub fn on<E, F>(&mut self, ev: E, callback: Rc<F>)
    where
        E: Event,
        F: IntoCallback<E::Arg>,
    {
        let el = EventListenerStorage {
            callback: AnonymousCallback::new(callback),
            next: None,
        };

        match self.listeners_header_tail.get(&ev.type_id()) {
            Some((_, tail)) => {
                self.listeners.push(el);
                self.listeners[*tail].next = Some(self.listeners.len() - 1);
            }
            None => {
                self.listeners.push(el);
                let self_index = self.listeners.len() - 1;
                self.listeners_header_tail
                    .insert(ev.type_id(), (self_index, self_index));
            }
        }
    }

    pub fn emit<E>(&self, ev: E, args: &E::Arg, event_flow: &mut EventFlow)
    where
        E: Event,
    {
        let mut current = self.listeners_header_tail.get(&ev.type_id()).map(|x| x.0);
        while let Some(cur) = current {
            let cb = &self.listeners[cur];
            cb.callback.try_call(args as _, event_flow).unwrap();
            current = cb.next;
        }
    }

    pub(crate) fn clear(&mut self) {
        self.listeners.clear();
        self.listeners_header_tail.clear();
    }

    pub(crate) fn events(&self) -> EventsIter {
        self.listeners_header_tail.keys()
    }
}
