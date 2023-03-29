use std::{any::Any, collections::HashMap, task::Waker};

use crate::Event;

pub(super) trait Visitor: Send + 'static {
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn emit(&mut self, el: &dyn Any, key: &dyn Any);
}

pub(super) struct Item<E, K> {
    counter: u32,
    pending: HashMap<u32, Option<Waker>>,
    ready: HashMap<u32, (E, K)>,
}

impl<E, K> Item<E, K>
where
    E: Event,
    K: Clone + Send + 'static,
{
    pub fn new() -> Self {
        Item {
            counter: 0,
            pending: HashMap::new(),
            ready: HashMap::new(),
        }
    }

    pub fn register(&mut self) -> u32 {
        loop {
            self.counter = self.counter.wrapping_add(1);
            let id = self.counter;

            if self.ready.contains_key(&id) {
                continue;
            }

            if let Some(old_waker) = self.pending.insert(id, None) {
                self.pending.insert(id, old_waker).unwrap();
            } else {
                break id;
            }
        }
    }

    pub fn update_waker(&mut self, id: u32, waker: Waker) {
        match self.pending.get_mut(&id) {
            Some(option) => *option = Some(waker),
            None => {
                #[cfg(debug_assertions)]
                panic!("inner error: id not exists");
            }
        }
    }

    pub fn finish(&mut self, ev: &E, key: &K) {
        for (id, waker) in self.pending.drain() {
            if let Some(waker) = waker {
                waker.wake();
            }
            debug_assert!(self.ready.insert(id, (ev.clone(), key.clone())).is_none());
        }
    }

    pub fn take(&mut self, id: u32) -> Option<(E, K)> {
        self.ready.remove(&id)
    }

    pub fn clear_by_id(&mut self, id: u32) {
        self.pending.remove(&id);
        self.ready.remove(&id);
    }
}

impl<E, K> Visitor for Item<E, K>
where
    E: Event,
    K: Clone + Send + 'static,
{
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn emit(&mut self, ev: &dyn Any, key: &dyn Any) {
        self.finish(
            ev.downcast_ref()
                .expect("inner error: element type mismatch"),
            key.downcast_ref().expect("inner error: key type mismatch"),
        );
    }
}
