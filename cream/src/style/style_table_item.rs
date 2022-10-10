use std::any::Any;

use crate::{
    data_driven::{constant_value::ConstantValue, Computed, Data, Watchable},
    map_rc::{MapRc, MapWeak},
};

#[derive(Debug, Clone, Copy)]
enum WatchableType {
    Computed,
    ConstantValue,
    Data,
}

pub struct TableItem {
    tag: WatchableType,
    storage: MapWeak<dyn Any>,
}

impl TableItem {
    pub fn get<D: 'static>(&self) -> MapRc<dyn Watchable<Data = D>> {
        let map_rc = self
            .storage
            .upgrade()
            .expect("The data must be come from the application where the table in");

        fn get_data<T: Watchable + 'static>(
            map_rc: &MapRc<dyn Any>,
        ) -> MapRc<dyn Watchable<Data = T::Data>> {
            MapRc::map(map_rc, |x| x.downcast_ref::<T>().unwrap() as _)
        }

        match self.tag {
            WatchableType::Computed => get_data::<Computed<D>>(&map_rc),
            WatchableType::ConstantValue => get_data::<ConstantValue<D>>(&map_rc),
            WatchableType::Data => get_data::<Data<D>>(&map_rc),
        }
    }
}

macro_rules! table_item_from {
    ($Struct:ident) => {
        impl<D: 'static> From<&MapRc<$Struct<D>>> for TableItem {
            fn from(from: &MapRc<$Struct<D>>) -> Self {
                TableItem {
                    tag: WatchableType::$Struct,
                    storage: MapRc::downgrade(&MapRc::map_to_any(from)),
                }
            }
        }
    };
}

table_item_from!(Computed);
table_item_from!(ConstantValue);
table_item_from!(Data);
