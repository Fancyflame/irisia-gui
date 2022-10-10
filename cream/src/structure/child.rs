use std::cell::RefCell;

use crate::map_rc::MapRc;

use super::dynamic_children::KeyedStorage;

pub enum Child<E> {
    Unkeyed { storage: MapRc<KeyedStorage> },
}

struct KeyedChild<K, E> {
    storage: MapRc<RefCell<KeyedStorage<K, E>>>,
    key: K,
}

trait KeyedStorageTrait<E: ?Sized> {
    fn get(&self) -> &E;
}

impl<K, E> KeyedStorageTrait<E> for KeyedChild<K, E> {
    fn get(&self) -> &E {
        self.storage
    }
}
