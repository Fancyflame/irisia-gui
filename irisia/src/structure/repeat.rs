use std::{
    collections::{hash_map::Entry, HashMap},
    hash::Hash,
};

use smallvec::SmallVec;

use crate::{dom::EMCreateCtx, Element};

use super::{StructureUpdater, VisitBy};

const MAX_TIME_TO_LIVE: u8 = 3;

pub struct Repeat<K, T> {
    map: HashMap<K, Item<T>>,
    order: SmallVec<[K; 5]>,
}

struct Item<T> {
    value: T,
    time_to_live: u8,
}

pub struct RepeatUpdater<I, Fm> {
    iter: I,
    map: Fm,
}

impl<K, T> VisitBy for Repeat<K, T>
where
    K: Hash + Eq,
    T: VisitBy,
{
    fn iter(&self) -> impl Iterator<Item = &dyn Element> {
        self.order
            .iter()
            .map(|k| self.map[k].value.iter())
            .flatten()
    }

    fn visit_mut(
        &mut self,
        mut f: impl FnMut(&mut dyn Element) -> crate::Result<()>,
    ) -> crate::Result<()> {
        for key in self.order.iter() {
            self.map
                .get_mut(key)
                .unwrap_or_else(|| unreachable!())
                .value
                .visit_mut(&mut f)?;
        }

        Ok(())
    }

    fn len(&self) -> usize {
        self.order.iter().map(|key| self.map[key].value.len()).sum()
    }
}

impl<K, T, I, Fm, Tu> StructureUpdater for RepeatUpdater<I, Fm>
where
    K: Hash + Eq + Clone + 'static,
    T: VisitBy + 'static,
    I: Iterator,
    Fm: for<'a> FnMut(I::Item) -> (K, Tu),
    Tu: StructureUpdater<Target = T>,
{
    type Target = Repeat<K, T>;

    fn create(self, ctx: &EMCreateCtx) -> Self::Target {
        let mut map = HashMap::new();
        let mut order = SmallVec::new();

        for (key, upd) in self.iter.map(self.map) {
            order.push(key.clone());
            map.insert(
                key,
                Item {
                    value: upd.create(ctx),
                    time_to_live: MAX_TIME_TO_LIVE,
                },
            );
        }

        Repeat { map, order }
    }

    fn update(self, target: &mut Self::Target, ctx: &EMCreateCtx) {
        target.order.clear();
        for (key, upd) in self.iter.map(self.map) {
            target.order.push(key.clone());

            match target.map.entry(key) {
                Entry::Occupied(mut occ) => {
                    let item = occ.get_mut();
                    assert_ne!(
                        item.time_to_live, MAX_TIME_TO_LIVE,
                        "some keys in the iterator is duplicated"
                    );
                    item.time_to_live = MAX_TIME_TO_LIVE;
                    upd.update(&mut item.value, ctx);
                }
                Entry::Vacant(vac) => {
                    vac.insert(Item {
                        value: upd.create(ctx),
                        time_to_live: MAX_TIME_TO_LIVE,
                    });
                }
            }
        }

        target
            .map
            .retain(|_, item| match item.time_to_live.checked_sub(1) {
                Some(ttl) => {
                    item.time_to_live = ttl;
                    true
                }
                None => false,
            });
    }
}
