use std::{
    collections::{hash_map::Entry, HashMap},
    hash::Hash,
};

use crate::{
    model::tools::DirtyPoints,
    prim_element::{EMCreateCtx, Element},
};

use super::{Model, VModel};

pub struct Repeat<I, F> {
    pub iter: I,
    pub key_fn: F,
}

struct Item<T> {
    used: bool,
    value: T,
}

impl<I, T, K, F> VModel for Repeat<I, F>
where
    I: Iterator<Item = T>,
    T: VModel,
    K: Hash + Eq + Clone + 'static,
    F: Fn(&T) -> K,
{
    const EXECUTE_POINTS: usize = T::EXECUTE_POINTS;
    type Storage = RepeatModel<K, T::Storage>;

    fn create(self, exec_point_offset: usize, ctx: &EMCreateCtx) -> Self::Storage {
        let capacity = self.iter.size_hint().0;
        let mut this = RepeatModel {
            map: HashMap::with_capacity(capacity),
            order: Vec::with_capacity(capacity),
        };

        for vmodel in self.iter {
            let key = (self.key_fn)(&vmodel);
            match this.map.entry(key.clone()) {
                Entry::Vacant(vac) => {
                    vac.insert(Item {
                        value: vmodel.create(exec_point_offset, ctx),
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

        this
    }

    fn update(self, storage: &mut Self::Storage, mut dp: DirtyPoints, ctx: &EMCreateCtx) {
        storage.order.clear();
        for vmodel in self.iter {
            let key = (self.key_fn)(&vmodel);
            match storage.map.entry(key.clone()) {
                Entry::Vacant(vac) => {
                    vac.insert(Item {
                        value: vmodel.create(dp.offset(), ctx),
                        used: true,
                    });
                }
                Entry::Occupied(mut occ) => {
                    let item = occ.get_mut();
                    item.used = true;
                    vmodel.update(&mut item.value, dp.fork(), ctx);
                }
            };
            storage.order.push(key.clone());
        }

        dp.consume(T::EXECUTE_POINTS);

        storage
            .map
            .retain(|_, item| std::mem::replace(&mut item.used, false));
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
