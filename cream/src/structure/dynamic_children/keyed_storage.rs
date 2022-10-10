use std::{collections::HashMap, hash::Hash};

use anyhow::Result;

use crate::{
    data_driven::{constant_value::ConstantValue, Watchable},
    map_rc::{MapRc, MapWeak},
    structure::{element::UpdateAndCreate, Element},
};

pub struct KeyedStorage<S, K, E> {
    map: HashMap<K, (E, ConstantValue<K>)>,
    create: fn(&S, MapWeak<dyn Watchable<Data = K>>) -> Result<E>,
}

impl<S, K, E> KeyedStorage<S, K, E>
where
    K: Eq + Hash + Clone,
    E: Element,
{
    pub fn new(create: fn(&S, MapWeak<dyn Watchable<Data = K>>) -> Result<E>) -> Self {
        KeyedStorage {
            map: HashMap::new(),
            create,
        }
    }

    pub fn get<A>(&mut self, slf: &S, key: &K) -> Result<&E>
    where
        E: UpdateAndCreate<A>,
    {
        if !self.map.contains_key(key) {
            self.map.insert(key.clone(), todo!());
        }
        Ok(&self.map.get(key).unwrap().0)
    }
}
