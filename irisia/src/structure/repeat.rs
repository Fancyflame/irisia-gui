use std::{
    cell::Cell,
    collections::{hash_map::Entry, HashMap},
    hash::Hash,
};

use smallvec::SmallVec;

use crate::{
    dep_watch::{bitset::UsizeArray, Bitset},
    Result,
};

use super::{tracert::TracertBase, StructureUpdateTo, Updating, VisitBy, VisitOn};

const MAX_TIME_TO_LIVE: u8 = 3;

pub struct Repeat<K, T, A: UsizeArray> {
    map: HashMap<K, Item<T>>,
    order: SmallVec<[K; 5]>,
    dependents: Cell<Bitset<A>>,
}

struct Item<T> {
    value: T,
    time_to_live: u8,
}

pub struct RepeatUpdater<Fi, Fm> {
    get_iter: Fi,
    map: Fm,
}

impl<K, T, A: UsizeArray> VisitBy for Repeat<K, T, A>
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

impl<K, T, Fi, I, Fm, Tu, A: UsizeArray> StructureUpdateTo<A> for RepeatUpdater<Fi, Fm>
where
    Self: VisitBy,
    K: Hash + Eq + Clone + 'static,
    T: VisitBy + 'static,
    Fi: FnOnce() -> I,
    I: Iterator,
    Fm: for<'a> FnMut(I::Item, TracertBase<'a, A>) -> (K, Tu),
    Tu: StructureUpdateTo<A, Target = T>,
{
    type Target = Repeat<K, T, A>;
    // 1 for the iterator expression
    const UPDATE_POINTS: u32 = 1 + Tu::UPDATE_POINTS;

    fn create(mut self, mut info: Updating<A>) -> Self::Target {
        let mut map = HashMap::new();
        let mut order = SmallVec::new();
        let dependents: Cell<Bitset<A>> = Default::default();

        let mut new_info = info.inherit(1, true);

        let tracert_base = TracertBase::new(new_info.stack, &dependents);
        let iterator = new_info.scoped(0, || {
            (self.get_iter)().map(|item| (self.map)(item, tracert_base))
        });

        for (key, upd) in iterator {
            order.push(key.clone());
            map.insert(
                key,
                Item {
                    value: upd.create(new_info.inherit(0, true)),
                    time_to_live: MAX_TIME_TO_LIVE,
                },
            );
        }

        Repeat {
            map,
            order,
            dependents,
        }
    }

    fn update(mut self, target: &mut Self::Target, mut info: Updating<A>) {
        if info.no_update::<Self>() {
            return;
        }

        info.step_if(0);

        let mut new_info = info.inherit(1, true);
        new_info.points.union(&target.dependents.take());

        let tracert_base = TracertBase::new(new_info.stack, &target.dependents);
        let iterator = new_info.scoped(0, || {
            (self.get_iter)().map(|item| (self.map)(item, tracert_base))
        });

        target.order.clear();
        for (key, upd) in iterator {
            target.order.push(key.clone());

            match target.map.entry(key) {
                Entry::Occupied(mut occ) => {
                    let item = occ.get_mut();
                    assert_ne!(
                        item.time_to_live, MAX_TIME_TO_LIVE,
                        "some keys in the iterator is duplicated"
                    );
                    item.time_to_live = MAX_TIME_TO_LIVE;
                    upd.update(&mut item.value, new_info.inherit(0, true));
                }
                Entry::Vacant(vac) => {
                    vac.insert(Item {
                        value: upd.create(new_info.inherit(0, true)),
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

        info.points
            .skip_range(info.update_point_offset..info.update_point_offset + Self::UPDATE_POINTS);
    }
}
