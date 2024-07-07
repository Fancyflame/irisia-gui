use std::{
    collections::{hash_map::Entry, HashMap},
    hash::Hash,
    rc::Rc,
};

use crate::{
    data_flow::{
        register::{register, Register},
        wire3, ReadWire,
    },
    el_model::EMCreateCtx,
};

use super::{StructureCreate, VisitBy};

const MAX_TIME_TO_LIVE: u8 = 3;

struct Repeat<K, T, Tree>(ReadWire<RepeatInner<K, T, Tree>>);

struct RepeatInner<K, T, Tree> {
    map: HashMap<K, Item<T, Tree>>,
    ctx: EMCreateCtx,
    order: Vec<K>,
}

struct Item<T, Tree> {
    iter_item: Rc<Register<T>>,
    tree: Tree,
    time_to_live: u8,
}

impl<K, T, Tree> VisitBy for Repeat<K, T, Tree>
where
    K: Hash + Eq + 'static,
    T: 'static,
    Tree: VisitBy,
{
    fn visit<V>(&self, v: &mut V) -> crate::Result<()>
    where
        V: super::Visitor,
    {
        let this = self.0.read();

        for key in &this.order {
            this.map[key].tree.visit(v)?;
        }

        Ok(())
    }
}

pub struct RepeatMutator<'a, K, T, Tree>(&'a mut RepeatInner<K, T, Tree>);

impl<'a, K, T, Tree> RepeatMutator<'a, K, T, Tree> {
    pub fn update<I, Upd, Fk, F>(self, iter: I, key_fn: Fk, content_fn: F)
    where
        K: Hash + Eq + Clone,
        T: 'static,
        I: Iterator<Item = T>,
        Fk: Fn(&T) -> K,
        F: Fn(ReadWire<T>) -> Upd,
        Upd: StructureCreate<Target = Tree>,
    {
        self.0
            .update(iter.map(|data| (key_fn(&data), data)), content_fn);
    }
}

pub fn repeat<K, T, Tree, F>(content_fn: F) -> impl StructureCreate
where
    K: Hash + Eq + Clone + 'static,
    T: 'static,
    Tree: VisitBy,
    F: Fn(RepeatMutator<K, T, Tree>) + 'static,
{
    move |ctx: &EMCreateCtx| {
        let ctx = ctx.clone();

        let w = wire3(
            move || {
                let rep = RepeatInner {
                    map: HashMap::new(),
                    ctx,
                    order: Vec::new(),
                };

                (rep, move |mut r| content_fn(RepeatMutator(&mut r)))
            },
            true,
        );

        Repeat(w)
    }
}

impl<K, T, Tree> RepeatInner<K, T, Tree> {
    fn update<I, Upd, F>(&mut self, iter: I, content_fn: F)
    where
        K: Hash + Eq + Clone,
        T: 'static,
        I: Iterator<Item = (K, T)>,
        F: Fn(ReadWire<T>) -> Upd,
        Upd: StructureCreate<Target = Tree>,
    {
        let RepeatInner { map, order, ctx } = self;

        order.clear();
        for (key, data) in iter {
            match map.entry(key.clone()) {
                Entry::Occupied(mut occ) => {
                    let item = occ.get_mut();
                    assert_ne!(
                        item.time_to_live, MAX_TIME_TO_LIVE,
                        "some keys in the iterator is duplicated"
                    );
                    item.time_to_live = MAX_TIME_TO_LIVE;
                    item.iter_item.set(data);
                }
                Entry::Vacant(vac) => {
                    let reg = register(data);
                    vac.insert(Item {
                        tree: content_fn(reg.clone()).create(ctx),
                        iter_item: reg,
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
