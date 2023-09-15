use std::{
    collections::{hash_map::Entry, HashMap},
    hash::Hash,
};

use smallvec::SmallVec;

use crate::{update_with::SpecificUpdate, Result, UpdateWith};

use super::{MapVisit, Visit, VisitLen, VisitMut};

pub struct Repeat<K, V>(Vec<(K, V)>);

impl<K, V> Repeat<K, V> {
    pub fn new(iter: impl Iterator<Item = (K, V)>) -> Self {
        Repeat(Vec::from_iter(iter))
    }
}

pub struct RepeatModel<K, T> {
    map: HashMap<K, T>,
    order: SmallVec<[K; 5]>,
}

// update

impl<K, T, U> UpdateWith<Repeat<K, U>> for RepeatModel<K, T>
where
    K: Hash + Eq + Clone + 'static,
    T: UpdateWith<U>,
{
    fn create_with(update: Repeat<K, U>) -> Self {
        let mut output = Self {
            order: SmallVec::from_iter(update.0.iter().map(|(key, _)| key.clone())),
            map: HashMap::from_iter(
                update
                    .0
                    .into_iter()
                    .map(|(key, val)| (key, T::create_with(val))),
            ),
        };

        output
    }

    fn update_with(&mut self, mut update: Repeat<K, U>, mut equality_matters: bool) -> bool {
        let mut insert = |key: K, value: U, equality_matters: &mut bool| match self.map.entry(key) {
            Entry::Occupied(mut model) => {
                *equality_matters &= model.get_mut().update_with(value, *equality_matters);
            }
            Entry::Vacant(model) => {
                model.insert(T::create_with(value));
                *equality_matters = false;
            }
        };

        let mut update_iter = update.0.into_iter();
        let mut old_keys = self.order.iter_mut();

        for ((k, v), vec_k) in (&mut update_iter).zip(&mut old_keys) {
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

        for (k, v) in update_iter {
            equality_matters = false;
            self.order.push(k.clone());
            insert(k, v, &mut equality_matters);
        }

        equality_matters
    }
}

// map

impl<K, V, Vis> MapVisit<Vis> for Repeat<K, V>
where
    V: MapVisit<Vis>,
{
    type Output = Repeat<K, V::Output>;
    fn map(self, visitor: &Vis) -> Self::Output {
        Repeat(Vec::from_iter(
            self.0.into_iter().map(|(key, val)| (key, val.map(visitor))),
        ))
    }
}

// visit

impl<K, T> VisitLen for Repeat<K, T> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<K, V, Vis> Visit<Vis> for Repeat<K, V>
where
    V: Visit<Vis>,
{
    fn visit(&self, visitor: &mut Vis) -> Result<()> {
        for x in self.0.iter() {
            x.1.visit(visitor)?;
        }
        Ok(())
    }
}

impl<K, V, Vis> VisitMut<Vis> for Repeat<K, V>
where
    V: VisitMut<Vis>,
{
    fn visit_mut(&mut self, visitor: &mut Vis) -> Result<()> {
        for x in self.0.iter_mut() {
            x.1.visit_mut(visitor)?;
        }
        Ok(())
    }
}

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

impl<K, V> SpecificUpdate for Repeat<K, V>
where
    V: SpecificUpdate,
{
    type UpdateTo = RepeatModel<K, V>;
}
