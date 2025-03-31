use std::{
    collections::{hash_map::Entry, HashMap},
    hash::Hash,
};

use crate::{
    model::tools::DirtyPoints,
    prim_element::{EMCreateCtx, Element},
};

use super::{Model, VModel};

pub struct Repeat<I> {
    pub iter: I,
}

struct Item<T> {
    used: bool,
    value: T,
}

impl<'a, I, T, K> VModel<'a> for Repeat<I>
where
    I: Iterator<Item = (K, T)>,
    K: Hash + Eq + Clone + 'static,
    T: VModel<'a>,
{
    const EXECUTE_POINTS: usize = T::EXECUTE_POINTS;
    type Storage = RepeatModel<K, T::Storage>;

    fn create(self, dp: &mut DirtyPoints<'a>, ctx: &EMCreateCtx) -> Self::Storage {
        let capacity = self.iter.size_hint().0;
        let mut this = RepeatModel {
            map: HashMap::with_capacity(capacity),
            order: Vec::with_capacity(capacity),
        };

        for (key, vmodel) in self.iter {
            match this.map.entry(key.clone()) {
                Entry::Vacant(vac) => {
                    vac.insert(Item {
                        value: vmodel.create(&mut dp.fork(), ctx),
                        used: false,
                    });
                    this.order.push(key.clone());
                }
                Entry::Occupied(_) => {
                    #[cfg(debug_assertions)]
                    panic!("each model must have unique key in repeat structure");
                }
            };
        }

        dp.consume(Self::EXECUTE_POINTS);
        this
    }

    fn update(self, storage: &mut Self::Storage, dp: &mut DirtyPoints<'a>, ctx: &EMCreateCtx) {
        storage.order.clear();
        for (key, vmodel) in self.iter {
            match storage.map.entry(key.clone()) {
                Entry::Vacant(vac) => {
                    vac.insert(Item {
                        value: vmodel.create(&mut dp.fork(), ctx),
                        used: true,
                    });
                }
                Entry::Occupied(mut occ) => {
                    let item = occ.get_mut();
                    item.used = true;
                    vmodel.update(&mut item.value, &mut dp.fork(), ctx);
                }
            };
            storage.order.push(key.clone());
        }

        storage
            .map
            .retain(|_, item| std::mem::replace(&mut item.used, false));
        dp.consume(Self::EXECUTE_POINTS);
    }
}

pub struct RepeatModel<K, T> {
    map: HashMap<K, Item<T>>,
    order: Vec<K>,
}

impl<K, T> Model for RepeatModel<K, T>
where
    K: Hash + Eq + 'static,
    T: Model,
{
    fn visit(&self, f: &mut dyn FnMut(Element)) {
        for key in &self.order {
            self.map[key].value.visit(f);
        }
    }
}
