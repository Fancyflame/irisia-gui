mod anony_watchable;

use std::{any::TypeId, collections::HashMap, rc::Rc};

use crate::data_driven::{DataSource, Watchable};

use self::anony_watchable::AnonymousWatchable;

use super::Style;

pub struct StyleTable {
    map: HashMap<TypeId, Rc<dyn AnonymousWatchable>>,
}

impl StyleTable {
    pub fn new() -> Self {
        StyleTable {
            map: HashMap::new(),
        }
    }

    pub fn set<W>(&mut self, style: &Rc<W>)
    where
        W: Watchable + 'static,
        W::Data: Style + 'static,
    {
        self.map.insert(TypeId::of::<W::Data>(), style.clone() as _);
    }

    pub fn get<St>(&self) -> Option<DataSource<St>>
    where
        St: Style + 'static,
    {
        let data = self.map.get(&TypeId::of::<St>())?.anonymous_get();

        if data.is::<St>() {
            Some(DataSource::map(data, |x| x.downcast_ref().unwrap()))
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }
}
