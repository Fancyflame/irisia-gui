use std::{
    collections::{hash_map::Entry, HashMap},
    hash::Hash,
};

use smallvec::SmallVec;

use crate::Result;

use super::{VisitBy, VisitLen, VisitMutBy};

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
    fn new() -> Self {
        Self {
            map: HashMap::new(),
            order: SmallVec::new(),
        }
    }

    fn update<I, U, F>(&mut self, iter: I, mut update: F)
    where
        I: Iterator<Item = (K, U)>,
        F: FnMut(UpdateNode<T>, &K, U),
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
                    update(UpdateNode::InPlace(&mut item.value), occ.key(), value);
                }
                Entry::Vacant(vac) => {
                    let mut place: Option<T> = None;
                    update(UpdateNode::NeedsOwnership(&mut place));

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

// visit

impl<K, T> VisitLen for Repeat<K, T>
where
    K: Hash + Eq,
    T: VisitLen,
{
    fn len(&self) -> usize {
        self.order.iter().map(|key| self.map[key].value.len()).sum()
    }
}

impl<K, T, V> VisitBy<V> for Repeat<K, T>
where
    K: Hash + Eq,
    T: VisitBy<V>,
{
    fn visit(&self, visitor: &mut V) -> Result<()> {
        for k in self.order.iter() {
            self.map[k].value.visit(visitor)?;
        }
        Ok(())
    }
}

impl<K, T, V> VisitMutBy<V> for Repeat<K, T>
where
    K: Hash + Eq,
    T: VisitMutBy<V>,
{
    fn visit_mut(&mut self, visitor: &mut V) -> Result<()> {
        for k in self.order.iter() {
            self.map.get_mut(k).unwrap().value.visit_mut(visitor)?;
        }
        Ok(())
    }
}
