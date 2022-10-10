mod style_table_item;

use std::{any::TypeId, collections::HashMap};

use crate::{data_driven::Watchable, map_rc::MapRc};

use style_table_item::TableItem;

pub trait Style {}

pub struct StyleTable {
    map: HashMap<TypeId, TableItem>,
}

impl StyleTable {
    pub fn new() -> Self {
        StyleTable {
            map: HashMap::new(),
        }
    }

    pub fn set<St, W>(&mut self, style: &MapRc<W>)
    where
        W: Watchable + 'static,
        TableItem: for<'r> From<&'r MapRc<W>>,
    {
        self.map.insert(TypeId::of::<W::Data>(), style.into());
    }

    pub fn get<St: Style + 'static>(&self) -> Option<MapRc<dyn Watchable<Data = St>>> {
        let item = self.map.get(&TypeId::of::<St>())?;
        Some(item.get::<St>())
    }
}
