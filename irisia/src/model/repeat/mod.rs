use std::{
    collections::{
        hash_map::{Entry, VacantEntry},
        HashMap,
    },
    hash::Hash,
};

use crate::{
    el_model::EMCreateCtx,
    hook::{Consumer, Provider},
};

use super::{
    iter::{ModelMapper, VisitModel},
    ModelCreateFn,
};

pub use {keyed::repeat_keyed, unkeyed::repeat_unkeyed};

mod keyed;
mod unkeyed;

pub struct Repeat<K, V, F>(Consumer<Inner<K, V, F>>);

struct Item<T> {
    value: T,
    used: bool,
}

struct Inner<K, V, F: ?Sized> {
    map: HashMap<K, Item<V>>,
    order: Vec<K>,
    ctx: EMCreateCtx,
    updator: F,
}

impl<M, K, V, F> VisitModel<M> for Repeat<K, V, F>
where
    M: ModelMapper,
    K: Hash + Eq + 'static,
    V: VisitModel<M> + 'static,
    F: 'static,
{
    fn visit(&self, f: &mut dyn FnMut(<M as ModelMapper>::MapRef<'_>)) {
        let this = self.0.borrow();
        for key in &this.order {
            this.map[key].value.visit(f);
        }
    }

    fn visit_mut(&self, f: &mut dyn FnMut(<M as ModelMapper>::MapMut<'_>)) {
        let mut borrowed = self.0.borrow_mut();
        let this = &mut *borrowed;

        for key in &this.order {
            this.map.get_mut(key).unwrap().value.visit_mut(f);
        }
    }
}

impl<K, V, F> Inner<K, V, F> {
    fn update<Data>(&mut self, dep: &Data)
    where
        F: Fn(&Data, RepeatUpdator<'_, K, V>, &EMCreateCtx) + 'static,
    {
        self.order.clear();
        for cache in &mut self.map {
            cache.1.used = false;
        }
        (self.updator)(
            dep,
            RepeatUpdator {
                map: &mut self.map,
                order: &mut self.order,
            },
            &self.ctx,
        );

        self.map.retain(|_, value| {
            if value.used {
                value.used = false;
                true
            } else {
                false
            }
        });
    }
}

struct RepeatUpdator<'a, K, V> {
    map: &'a mut HashMap<K, Item<V>>,
    order: &'a mut Vec<K>,
}

impl<K, V> RepeatUpdator<'_, K, V>
where
    K: Hash + Eq + Clone,
{
    fn push(&mut self, key: K) -> Inserting<K, V> {
        match self.map.entry(key.clone()) {
            Entry::Occupied(occ) => {
                if occ.get().used {
                    panic!("cannot update a model with same key twice in a repetitive structure");
                }
                self.order.push(key);

                let item = occ.into_mut();
                item.used = true;
                Inserting::Occupied(&mut item.value)
            }
            Entry::Vacant(vac) => {
                self.order.push(key);
                Inserting::Vacant(InsertingVacant(vac))
            }
        }
    }
}

enum Inserting<'a, K, V> {
    Vacant(InsertingVacant<'a, K, V>),
    Occupied(&'a mut V),
}

struct InsertingVacant<'a, K, V>(VacantEntry<'a, K, Item<V>>);

impl<'a, K, V> InsertingVacant<'a, K, V> {
    fn insert(self, value: V) {
        self.0.insert(Item { value, used: true });
    }
}

fn repeat<M, K, V, F, D>(updator: F, dep_iter: D) -> impl ModelCreateFn<M>
where
    M: ModelMapper,
    K: Hash + Eq + Clone + 'static,
    V: VisitModel<M> + 'static,
    F: Fn(&D::Data, RepeatUpdator<K, V>, &EMCreateCtx) + Clone + 'static,
    D: Provider + Clone + 'static,
{
    move |ctx| {
        Repeat(Consumer::new(
            Inner {
                map: HashMap::new(),
                order: Vec::new(),
                updator: updator.clone(),
                ctx: ctx.clone(),
            },
            Inner::update,
            dep_iter.clone(),
        ))
    }
}
