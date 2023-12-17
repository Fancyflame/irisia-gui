use std::{
    collections::{hash_map::Entry, HashMap},
    hash::Hash,
};

use smallvec::SmallVec;

use crate::Result;

use super::{UpdateNode, VisitBy, VisitOn};

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

impl<K, T> Repeat<K, T>
where
    K: Clone + Hash + Eq + 'static,
{
    pub fn new<I, U, F>(iter: I, update: F) -> Self
    where
        I: Iterator<Item = (K, U)>,
        F: Fn(U) -> T,
    {
        let iter_size = iter.size_hint().0;
        let mut map = HashMap::with_capacity(iter_size);
        let mut order = SmallVec::with_capacity(iter_size);

        for (key, value) in iter {
            order.push(key.clone());
            map.insert(
                key,
                Item {
                    value: update(value),
                    time_to_live: MAX_TIME_TO_LIVE,
                },
            );
        }

        Self { map, order }
    }

    pub fn update_tree<F>(&mut self, update: F)
    where
        F: Fn(&mut T),
    {
        for key in &self.order {
            update(&mut self.map.get_mut(key).unwrap().value);
        }
    }

    pub fn update_data<I, U, F>(&mut self, iter: I, update: F)
    where
        I: Iterator<Item = (K, U)>,
        F: Fn(UpdateNode<T>, U),
    {
        self.order.clear();

        for (k, value) in iter {
            self.order.push(k.clone());
            match self.map.entry(k) {
                Entry::Occupied(mut occ) => {
                    let item = occ.get_mut();
                    assert_ne!(
                        item.time_to_live, MAX_TIME_TO_LIVE,
                        "some keys in the iterator is duplicated"
                    );
                    item.time_to_live = MAX_TIME_TO_LIVE;
                    update(UpdateNode::NeedsUpdate(&mut item.value), value);
                }
                Entry::Vacant(vac) => {
                    let mut place: Option<T> = None;
                    update(UpdateNode::NeedsInit(&mut place), value);

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
