use std::{collections::HashMap, task::Waker};

use crate::{event::EventMetadata, Event};

enum Ltnr<E> {
    Pending(Option<Waker>),
    Ready { event: E, metadata: EventMetadata },
}

pub(super) struct Item<E> {
    counter: u32,
    listeners: HashMap<u32, Ltnr<E>>,
}

impl<E: Event> Item<E> {
    pub fn new() -> Self {
        Item {
            counter: 0,
            listeners: Default::default(),
        }
    }

    pub fn register(&mut self) -> u32 {
        loop {
            self.counter = self.counter.wrapping_add(1);
            let id = self.counter;

            if let Some(old) = self.listeners.insert(id, Ltnr::Pending(None)) {
                self.listeners.insert(id, old).unwrap();
            } else {
                break id;
            }
        }
    }

    pub fn finish(&mut self, ev: E, metadata: EventMetadata) {
        for ltnr in self.listeners.values_mut() {
            if let Ltnr::Pending(waker_option) = ltnr {
                if let Some(waker) = waker_option.take() {
                    waker.wake();
                }
                *ltnr = Ltnr::Ready {
                    event: ev.clone(),
                    metadata,
                };
            }
        }
    }

    pub fn poll(&mut self, id: u32, waker: Waker) -> Option<(E, EventMetadata)> {
        match self.listeners.get_mut(&id) {
            Some(Ltnr::Ready { .. }) => match self.listeners.remove(&id).unwrap() {
                Ltnr::Ready { event, metadata } => Some((event, metadata)),
                _ => unreachable!(),
            },

            Some(Ltnr::Pending(option)) => {
                option.replace(waker);
                None
            }

            None => {
                if cfg!(debug_assertions) {
                    panic!("inner error: cannot call `take` on this id");
                } else {
                    None
                }
            }
        }
    }

    pub fn clear_by_id(&mut self, id: u32) {
        self.listeners.remove(&id);
    }
}
