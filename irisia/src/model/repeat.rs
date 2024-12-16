use std::{
    collections::{hash_map::Entry, HashMap},
    hash::Hash,
};

use crate::el_model::EMCreateCtx;

use super::{
    iter::{ModelMapper, VisitModel},
    VModel,
};

pub struct Repeat<I>(pub I);

struct Item<T> {
    used: bool,
    value: T,
}

impl<I, K, T> VModel for Repeat<I>
where
    I: Iterator<Item = (K, T)> + Clone,
    K: Hash + Eq + Clone + 'static,
    T: VModel,
{
    type Storage = RepeatModel<K, T::Storage>;
    fn create(&self, ctx: &EMCreateCtx) -> Self::Storage {
        let size_hint = self.0.size_hint().0;
        let mut this = RepeatModel {
            map: HashMap::with_capacity(size_hint),
            order: Vec::with_capacity(size_hint),
        };

        for (key, vmodel) in self.0.clone() {
            match this.map.entry(key.clone()) {
                Entry::Vacant(vac) => {
                    vac.insert(Item {
                        value: vmodel.create(ctx),
                        used: false,
                    });
                    this.order.push(key);
                }
                Entry::Occupied(_) => {
                    #[cfg(debug_assertions)]
                    panic!("each model must have unique key in repeat structure");
                }
            };
        }

        this
    }

    fn update(&self, storage: &mut Self::Storage, ctx: &EMCreateCtx) {
        storage.order.clear();
        for (key, vmodel) in self.0.clone() {
            match storage.map.entry(key.clone()) {
                Entry::Vacant(vac) => {
                    vac.insert(Item {
                        value: vmodel.create(ctx),
                        used: true,
                    });
                }
                Entry::Occupied(mut occ) => {
                    let item = occ.get_mut();
                    item.used = true;
                    vmodel.update(&mut item.value, ctx);
                }
            };
            storage.order.push(key);
        }

        storage
            .map
            .retain(|_, item| std::mem::replace(&mut item.used, false));
    }
}

fn vec_to_repeat<K: Clone, T>(vec: &Vec<(K, T)>) -> Repeat<impl Iterator<Item = (K, &T)> + Clone> {
    Repeat(vec.iter().map(|(k, v)| (k.clone(), v)))
}

impl<K, T> VModel for Vec<(K, T)>
where
    K: Hash + Eq + Clone + 'static,
    T: VModel,
{
    type Storage = RepeatModel<K, T::Storage>;
    fn create(&self, ctx: &EMCreateCtx) -> Self::Storage {
        vec_to_repeat(self).create(ctx)
    }
    fn update(&self, storage: &mut Self::Storage, ctx: &EMCreateCtx) {
        vec_to_repeat(self).update(storage, ctx);
    }
}

pub struct RepeatModel<K, T> {
    map: HashMap<K, Item<T>>,
    order: Vec<K>,
}

impl<M, K, T> VisitModel<M> for RepeatModel<K, T>
where
    M: ModelMapper,
    K: Hash + Eq,
    T: VisitModel<M>,
{
    fn visit(&self, f: &mut dyn FnMut(<M as ModelMapper>::MapRef<'_>)) {
        for key in &self.order {
            self.map[key].value.visit(f);
        }
    }
    fn visit_mut(&mut self, f: &mut dyn FnMut(<M as ModelMapper>::MapMut<'_>)) {
        for key in &self.order {
            self.map.get_mut(key).unwrap().value.visit_mut(f);
        }
    }
}
