use std::{
    collections::{hash_map::Entry, HashMap},
    hash::Hash,
};

use crate::{
    data_flow::{wire3, ReadWire},
    el_model::EMCreateCtx,
};

use super::{StructureCreate, VisitBy};

const MAX_TIME_TO_LIVE: u8 = 3;

struct Repeat<K, T>(ReadWire<RepeatInner<K, T>>);

struct RepeatInner<K, T> {
    map: HashMap<K, Item<T>>,
    order: Vec<K>,
}

struct Item<T> {
    value: T,
    time_to_live: u8,
}

impl<K, T> VisitBy for Repeat<K, T>
where
    K: Hash + Eq + 'static,
    T: VisitBy,
{
    fn visit<V>(&self, v: &mut V) -> crate::Result<()>
    where
        V: super::Visitor,
    {
        let this = self.0.read();

        for key in &this.order {
            this.map[key].value.visit(v)?;
        }

        Ok(())
    }

    fn len(&self) -> usize {
        let this = self.0.read();

        let len = this.order.iter().map(|key| this.map[key].value.len()).sum();

        len
    }
}

pub fn repeat<K, T, F, Fk, Upd>(
    data_vec: ReadWire<Vec<T>>,
    map_key: Fk,
    content_fn: F,
) -> impl StructureCreate
where
    T: 'static,
    K: Hash + Eq + Clone + 'static,
    Fk: Fn(usize, &T) -> K + 'static,
    F: Fn(&T) -> Upd + 'static,
    Upd: StructureCreate,
{
    move |ctx: &EMCreateCtx| {
        let ctx = ctx.clone();

        let w = wire3(move || {
            let mut map = HashMap::with_capacity(data_vec.read().len());
            let mut order = Vec::with_capacity(data_vec.read().len());

            for (index, data) in data_vec.read().iter().enumerate() {
                let key = map_key(index, data);

                map.insert(
                    key.clone(),
                    Item {
                        value: content_fn(data).create(&ctx),
                        time_to_live: MAX_TIME_TO_LIVE - 1,
                    },
                );

                order.push(key);
            }

            (RepeatInner { map, order }, move |r| {
                r.update_map(
                    &map_key,
                    |key| content_fn(key).create(&ctx),
                    &data_vec.read(),
                )
            })
        });

        Repeat(w)
    }
}

impl<K, Tree> RepeatInner<K, Tree>
where
    K: Hash + Eq + Clone,
{
    fn update_map<T, Fk, F>(&mut self, key_fn: Fk, content_fn: F, data_vec: &Vec<T>)
    where
        Fk: Fn(usize, &T) -> K,
        F: Fn(&T) -> Tree,
    {
        let RepeatInner { map, order } = self;
        order.clear();

        for (index, data) in data_vec.iter().enumerate() {
            let key = key_fn(index, data);

            match map.entry(key.clone()) {
                Entry::Occupied(mut occ) => {
                    let item = occ.get_mut();
                    assert_ne!(
                        item.time_to_live, MAX_TIME_TO_LIVE,
                        "some keys in the iterator is duplicated"
                    );
                    item.time_to_live = MAX_TIME_TO_LIVE;
                }
                Entry::Vacant(vac) => {
                    vac.insert(Item {
                        value: content_fn(data),
                        time_to_live: MAX_TIME_TO_LIVE,
                    });
                }
            }

            order.push(key);
        }

        map.retain(|_, item| match item.time_to_live.checked_sub(1) {
            Some(ttl) => {
                item.time_to_live = ttl;
                true
            }
            None => false,
        });
    }
}
