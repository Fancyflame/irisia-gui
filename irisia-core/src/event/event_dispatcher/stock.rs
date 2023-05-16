use std::{
    any::{Any, TypeId},
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
    task::Waker,
};

use crate::{event::EventMetadata, Event};

use super::maybe_confirmed::{AllConfirmedPermits, MaybeConfirmed};

pub(super) struct EventListenerStock {
    stocks: HashMap<TypeId, Box<dyn Any + Send>>,
    wait_lock: Arc<MaybeConfirmed>,
}

impl EventListenerStock {
    pub fn new(wait_lock: Arc<MaybeConfirmed>) -> Self {
        EventListenerStock {
            stocks: HashMap::new(),
            wait_lock,
        }
    }

    pub fn get<E: Event>(&mut self) -> Option<&mut Row<E>> {
        self.stocks.get_mut(&TypeId::of::<E>()).map(|any| {
            any.downcast_mut()
                .unwrap_or_else(|| inner_error!("cannot downcast to item"))
        })
    }

    pub fn get_or_insert<E: Event>(&mut self) -> &mut Row<E> {
        self.stocks
            .entry(TypeId::of::<E>())
            .or_insert_with(|| Box::new(Row::<E>::new(self.wait_lock.clone())))
            .downcast_mut()
            .unwrap_or_else(|| inner_error!("cannot downcast to item"))
    }

    pub fn get_exist<E: Event>(&mut self) -> &mut Row<E> {
        match self.get() {
            Some(item) => item,
            None => {
                inner_error!("item not exits, but expected does");
            }
        }
    }
}

enum Ltnr<E> {
    Pending {
        waker: Option<Waker>,
        increased_permits: bool,
    },
    Ready {
        event: E,
        metadata: EventMetadata,
    },
}

pub(super) struct Row<E> {
    id_generator: u32,
    confirmed_count: u32,
    listeners: HashMap<u32, Ltnr<E>>,
    wait_lock: Arc<MaybeConfirmed>,
}

impl<E: Event> Row<E> {
    fn new(wait_lock: Arc<MaybeConfirmed>) -> Self {
        Row {
            id_generator: 0,
            confirmed_count: 0,
            listeners: Default::default(),
            wait_lock,
        }
    }

    pub fn register(&mut self, increased_permits: bool) -> u32 {
        loop {
            let id = self.id_generator;
            self.id_generator = self.id_generator.wrapping_add(1);

            if increased_permits {
                self.confirmed_count += 1;
            }

            if let Entry::Vacant(place) = self.listeners.entry(id) {
                place.insert(Ltnr::Pending {
                    waker: None,
                    increased_permits,
                });
                break id;
            }
        }
    }

    pub fn finish(&mut self, ev: E, metadata: EventMetadata, mut all_cfm_pmt: AllConfirmedPermits) {
        for ltnr in self.listeners.values_mut() {
            if let Ltnr::Pending {
                waker: waker_option,
                increased_permits: _,
            } = ltnr
            {
                if let Some(waker) = waker_option.take() {
                    waker.wake();
                }

                *ltnr = Ltnr::Ready {
                    event: ev.clone(),
                    metadata,
                };
            }
        }

        all_cfm_pmt.cancel_many(self.confirmed_count);
        self.confirmed_count = 0;
    }

    pub fn poll(&mut self, id: u32, waker: Waker) -> Option<(E, EventMetadata)> {
        match self.listeners.get_mut(&id) {
            Some(Ltnr::Ready { .. }) => match self.listeners.remove(&id).unwrap() {
                Ltnr::Ready { event, metadata } => Some((event, metadata)),
                _ => unreachable!(),
            },

            Some(Ltnr::Pending { waker: option, .. }) => {
                option.replace(waker);
                None
            }

            None => {
                if cfg!(debug_assertions) {
                    inner_error!("cannot call `take` on this id");
                } else {
                    None
                }
            }
        }
    }

    pub fn clear_by_id(&mut self, id: u32) {
        if let Some(Ltnr::Pending {
            increased_permits: true,
            ..
        }) = self.listeners.remove(&id)
        {
            self.confirmed_count -= 1;
            self.wait_lock.cancel_one();
        }
    }
}

impl<E> Drop for Row<E> {
    fn drop(&mut self) {
        self.wait_lock.cancel_many(self.confirmed_count);
    }
}
