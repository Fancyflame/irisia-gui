use std::{cell::Ref, ops::Deref};

use crate::map_rc::MapRc;

/// Implement this trait says that it can be a dependent of a
/// data source.
pub trait DepNode {
    fn on_update(&self);
}

pub trait Watchable<D> {
    fn get(&self) -> DataSource<D>;
    fn subscribe(&self, sub: &MapRc<dyn DepNode>);
}

enum DataSourceEnum<'a, T> {
    Ref(Ref<'a, T>),
    Borrow(&'a T),
}
pub struct DataSource<'a, T>(DataSourceEnum<'a, T>);

impl<'a, T> Deref for DataSource<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match &self.0 {
            DataSourceEnum::Borrow(b) => b,
            DataSourceEnum::Ref(r) => &*r,
        }
    }
}

impl<'a, T> From<&'a T> for DataSource<'a, T> {
    fn from(f: &'a T) -> Self {
        DataSource(DataSourceEnum::Borrow(f))
    }
}

impl<'a, T> From<Ref<'a, T>> for DataSource<'a, T> {
    fn from(f: Ref<'a, T>) -> Self {
        DataSource(DataSourceEnum::Ref(f))
    }
}
