use std::{
    collections::{hash_map::Entry, HashMap},
    hash::Hash,
};

use smallvec::SmallVec;

use crate::Result;

use super::{VisitBy, VisitOn};

const MAX_TIME_TO_LIVE: u8 = 5;

pub struct Repeat<K, T> {
    map: HashMap<K, Item<T>>,
    order: SmallVec<[K; 5]>,
}

struct Item<T> {
    value: T,
    time_to_live: u8,
}

// update

pub enum UpdateNode<'a, T> {
    NeedsInit(&'a mut Option<T>),
    NeedsUpdate(&'a mut T),
}

impl<K, T> Repeat<K, T>
where
    K: Clone + Hash + Eq + 'static,
{
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            order: SmallVec::new(),
        }
    }

    pub fn update_tree<I, U, F>(&mut self, mut update: F)
    where
        F: FnMut(&mut T),
    {
        for key in &self.order {
            update(&mut self.map.get_mut(key).unwrap().value);
        }
    }

    pub fn update_data<I, U, F>(&mut self, iter: I, mut update: F)
    where
        I: Iterator<Item = (K, U)>,
        F: FnMut(UpdateNode<T>, K, U),
    {
        self.order.clear();

        for (k, value) in iter {
            self.order.push(k.clone());
            match self.map.entry(k.clone()) {
                Entry::Occupied(mut occ) => {
                    let item = occ.get_mut();
                    assert_ne!(
                        item.time_to_live, MAX_TIME_TO_LIVE,
                        "some keys in the iterator is duplicated"
                    );
                    item.time_to_live = MAX_TIME_TO_LIVE;
                    update(UpdateNode::NeedsUpdate(&mut item.value), k, value);
                }
                Entry::Vacant(vac) => {
                    let mut place: Option<T> = None;
                    update(UpdateNode::NeedsInit(&mut place), k, value);

                    vac.insert(Item {
                        value: place.expect("new node was not inserted"),
                        time_to_live: MAX_TIME_TO_LIVE,
                    });
                }
            }
        }

        self.map
            .retain(|_, item| match item.time_to_live.checked_sub(1) {
                Some(ttl) => {
                    item.time_to_live = ttl;
                    true
                }
                None => false,
            });
    }
}

impl<K, T> VisitBy for Repeat<K, T>
where
    K: Hash + Eq,
    T: VisitBy,
{
    fn visit_by<V>(&self, visitor: &mut V) -> Result<()>
    where
        V: VisitOn,
    {
        for k in self.order.iter() {
            self.map[k].value.visit_by(visitor)?;
        }
        Ok(())
    }

    fn len(&self) -> usize {
        self.order.iter().map(|key| self.map[key].value.len()).sum()
    }
}
