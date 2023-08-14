use std::collections::hash_map::Entry;
use std::{collections::HashMap, hash::Hash};

use smallvec::SmallVec;

use crate::update_with::SpecificUpdate;
use crate::Result;

use super::{MapVisit, UpdateWith, Visit, VisitLen, VisitMut};

pub struct Repeat<I> {
    iter: I,
}

impl<I: Iterator> Repeat<I> {
    pub fn new(iter: I) -> Self {
        Repeat { iter }
    }
}

pub struct RepeatModel<K, T> {
    map: HashMap<K, T>,
    order: SmallVec<[K; 5]>,
}

// map

pub struct MapIter<I, V> {
    iter: I,
    map_visit: V,
}

impl<I, K, T, V> Iterator for MapIter<I, V>
where
    I: Iterator<Item = (K, T)>,
    T: MapVisit<V>,
{
    type Item = (K, T::Output);
    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(k, val)| (k, val.map(&self.map_visit)))
    }
}

impl<I, K, T, V> MapVisit<V> for Repeat<I>
where
    I: Iterator<Item = (K, T)>,
    T: MapVisit<V>,
    V: Clone,
{
    type Output = Repeat<MapIter<I, V>>;
    fn map(self, visitor: &V) -> Self::Output {
        Repeat {
            iter: MapIter {
                iter: self.iter,
                map_visit: visitor.clone(),
            },
        }
    }
}

// visit

impl<K, T> VisitLen for RepeatModel<K, T> {
    fn len(&self) -> usize {
        self.order.len()
    }
}

impl<K, T, V> Visit<V> for RepeatModel<K, T>
where
    K: Hash + Eq,
    T: Visit<V>,
{
    fn visit(&self, visitor: &mut V) -> Result<()> {
        for k in self.order.iter() {
            self.map[&k].visit(visitor)?;
        }
        Ok(())
    }
}

impl<K, T, V> VisitMut<V> for RepeatModel<K, T>
where
    K: Hash + Eq,
    T: VisitMut<V>,
{
    fn visit_mut(&mut self, visitor: &mut V) -> Result<()> {
        for k in self.order.iter() {
            self.map.get_mut(k).unwrap().visit_mut(visitor)?;
        }
        Ok(())
    }
}

// update

impl<K, I, T, U> UpdateWith<Repeat<I>> for RepeatModel<K, T>
where
    K: Hash + Eq + Clone + 'static,
    I: Iterator<Item = (K, U)>,
    T: UpdateWith<U>,
{
    fn create_with(update: Repeat<I>) -> Self {
        let mut output = Self {
            order: SmallVec::new(),
            map: HashMap::new(),
        };

        for (k, v) in update.iter {
            output.order.push(k.clone());
            output.map.insert(k, T::create_with(v));
        }

        output
    }

    fn update_with(&mut self, mut update: Repeat<I>, mut equality_matters: bool) -> bool {
        let mut insert = |key: K, value: U, equality_matters: &mut bool| match self.map.entry(key) {
            Entry::Occupied(mut model) => {
                *equality_matters &= model.get_mut().update_with(value, *equality_matters);
            }
            Entry::Vacant(model) => {
                model.insert(T::create_with(value));
                *equality_matters = false;
            }
        };

        let mut old_keys = self.order.iter_mut();
        for ((k, v), vec_k) in (&mut update.iter).zip(&mut old_keys) {
            if equality_matters {
                equality_matters = k == *vec_k;
            }
            *vec_k = k.clone();

            insert(k, v, &mut equality_matters);
        }

        if old_keys.len() != 0 {
            let old_key_len = old_keys.len();
            self.order.truncate(self.order.len() - old_key_len);
            return false;
        }

        for (k, v) in update.iter {
            equality_matters = false;
            self.order.push(k.clone());
            insert(k, v, &mut equality_matters);
        }

        equality_matters
    }
}

impl<I, K, V> SpecificUpdate for Repeat<I>
where
    I: Iterator<Item = (K, V)>,
    V: SpecificUpdate,
{
    type UpdateTo = RepeatModel<K, V>;
}
