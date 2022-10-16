use std::{
    cell::RefCell,
    collections::HashMap,
    hash::Hash,
    rc::{Rc, Weak},
};

use anyhow::Result;

use crate::{
    data_driven::const_data::ConstData,
    structure::element::{ElementHandle, RcHandle},
};

pub struct KeyedStorage<S, K, E> {
    map: HashMap<K, (Rc<ConstData<K>>, RcHandle<E>)>,
    weak: Weak<S>,
    create: fn(&Weak<S>, Rc<ConstData<K>>) -> Result<E>,
}

impl<S, K, E> KeyedStorage<S, K, E>
where
    K: Eq + Hash + Clone,
{
    pub fn new(slf: Weak<S>, create: fn(&Weak<S>, Rc<ConstData<K>>) -> Result<E>) -> Self {
        KeyedStorage {
            map: HashMap::new(),
            weak: slf,
            create,
        }
    }

    pub fn get(&mut self, key: &K) -> Result<RcHandle<E>> {
        if !self.map.contains_key(key) {
            let data = ConstData::new(key.clone());
            let ele = Rc::new(RefCell::new((self.create)(&self.weak, data.clone())?));
            self.map.insert(key.clone(), (data, ele));
        }
        Ok(self.map.get(key).unwrap().1.clone())
    }
}

pub(super) trait KeyedStoageTrait {
    fn get(&self) -> ElementHandle;
}
