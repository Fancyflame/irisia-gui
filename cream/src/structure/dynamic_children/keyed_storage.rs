use std::{collections::HashMap, hash::Hash};

use anyhow::Result;

use crate::structure::{element::UpdateAndCreate, Element};

pub struct KeyedStorage<K, E> {
    map: HashMap<K, E>,
}

impl<K, E> KeyedStorage<K, E>
where
    K: Eq + Hash + Clone,
    E: Element,
{
    pub fn new() -> Self {
        KeyedStorage {
            map: HashMap::new(),
        }
    }

    pub fn update<A>(&mut self, key: &K, args: A) -> Result<&E>
    where
        E: UpdateAndCreate<A>,
    {
        if self.map.contains_key(key) {
            let cache = self.map.get_mut(key).unwrap();
            E::update(cache, args)?;
            Ok(&*cache)
        } else {
            self.map.insert(key.clone(), E::create(args)?);
            Ok(self.map.get(key).unwrap())
        }
    }
}
