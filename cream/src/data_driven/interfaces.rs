use std::{cell::Ref, ops::Deref, rc::Rc};

/// Implement this trait says that it can be a dependent of a
/// data source.
pub trait DepNode {
    fn on_update(&self);
}

pub trait Watchable {
    type Data;
    fn get(&self) -> DataSource<Self::Data>;
    fn subscribe(&self, sub: &Rc<dyn DepNode>);
}

enum DataSourceEnum<'a, T: ?Sized> {
    Ref(Ref<'a, T>),
    Borrow(&'a T),
}

pub struct DataSource<'a, T: ?Sized>(DataSourceEnum<'a, T>);

impl<'a, T: ?Sized> DataSource<'a, T> {
    pub fn map<F, U>(orig: Self, func: F) -> DataSource<'a, U>
    where
        F: FnOnce(&T) -> &U,
        U: ?Sized + 'a,
    {
        match orig.0 {
            DataSourceEnum::Ref(r) => Ref::map(r, func).into(),
            DataSourceEnum::Borrow(b) => func(b).into(),
        }
    }
}

impl<T: ?Sized> Deref for DataSource<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match &self.0 {
            DataSourceEnum::Borrow(b) => b,
            DataSourceEnum::Ref(r) => &*r,
        }
    }
}

impl<'a, T: ?Sized> From<&'a T> for DataSource<'a, T> {
    fn from(f: &'a T) -> Self {
        DataSource(DataSourceEnum::Borrow(f))
    }
}

impl<'a, T: ?Sized> From<Ref<'a, T>> for DataSource<'a, T> {
    fn from(f: Ref<'a, T>) -> Self {
        DataSource(DataSourceEnum::Ref(f))
    }
}
