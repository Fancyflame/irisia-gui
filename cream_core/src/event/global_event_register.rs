use std::{any::TypeId, collections::HashMap};

use smallvec::SmallVec;

use crate::{
    event::{event_state::wrap::WrappedEvents, Event, EventFlow},
    primary::{Point, Region},
};

type Listeners = (WrappedEvents, Region);

struct Classified {
    listeners: SmallVec<[Listeners; 4]>,
    is_classified: bool,
}

pub(crate) struct SystemEventRegister {
    all: Vec<(WrappedEvents, Region)>,
    classified: HashMap<TypeId, Classified>,
}

impl SystemEventRegister {
    pub fn new() -> Self {
        SystemEventRegister {
            all: Vec::new(),
            classified: HashMap::new(),
        }
    }

    pub fn listen_list(&mut self, el: WrappedEvents, region: Region) {
        self.all.push((el, region));
    }

    pub fn emit<E: Event>(&mut self, event: &E, point: Option<Point>) {
        let event_id = TypeId::of::<E>();
        match self.classified.get_mut(&event_id) {
            Some(cls) if cls.is_classified => {
                EventFlow::call_multiple(cls.listeners.iter(), event, point);
            }

            Some(cls) => {
                cls.listeners.clear();

                for el in self.all.iter() {
                    if el.0.is_related::<E>() {
                        cls.listeners.push(el.clone());
                    }
                }

                cls.is_classified = true;
                EventFlow::call_multiple(cls.listeners.iter(), event, point);
            }

            None => {
                let mut vec = SmallVec::new();

                for el in self.all.iter() {
                    if el.0.is_related::<E>() {
                        vec.push(el.clone());
                    }
                }

                EventFlow::call_multiple(vec.iter(), event, point);

                self.classified.insert(
                    event_id,
                    Classified {
                        listeners: vec,
                        is_classified: true,
                    },
                );
            }
        }
    }

    pub fn clear(&mut self) {
        self.all.clear();
        for l in self.classified.values_mut() {
            l.listeners.clear();
            l.is_classified = false;
        }
    }
}
