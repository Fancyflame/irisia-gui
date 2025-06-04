use std::{
    collections::{HashMap, hash_map::Entry},
    hash::Hash,
};

use crate::{model::ModelCreateCtx, prim_element::Element};

use crate::model::{Model, VModel};

impl<K, T, Cd> VModel<Cd> for Vec<(K, T)>
where
    K: Hash + Eq + Clone + 'static,
    T: VModel<Cd>,
{
    type Storage = RepeatModel<K, T::Storage>;

    fn create(&self, ctx: &ModelCreateCtx) -> Self::Storage {
        let mut this = RepeatModel {
            map: HashMap::with_capacity(self.len()),
            order: Vec::with_capacity(self.len()),
        };

        for (key, vmodel) in self.iter() {
            match this.map.entry(key.clone()) {
                Entry::Vacant(vac) => {
                    vac.insert(Item {
                        value: vmodel.create(ctx),
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

    fn update(&self, storage: &mut Self::Storage, ctx: &ModelCreateCtx) {
        storage.order.clear();
        for (key, vmodel) in self.iter() {
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
            storage.order.push(key.clone());
        }

        storage
            .map
            .retain(|_, item| std::mem::replace(&mut item.used, false));
    }
}

pub struct RepeatModel<K, T> {
    map: HashMap<K, Item<T>>,
    order: Vec<K>,
}

struct Item<T> {
    used: bool,
    value: T,
}

impl<K, T, Cd> Model<Cd> for RepeatModel<K, T>
where
    K: Hash + Eq + 'static,
    T: Model<Cd>,
{
    fn visit(&self, f: &mut dyn FnMut(Element, Cd)) {
        for key in &self.order {
            self.map[key].value.visit(f);
        }
    }
}
