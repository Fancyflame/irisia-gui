use crate::model::component::definition::Definition;
pub use extend::*;

pub mod extend;
pub mod macro_utils;
pub mod signals;

pub trait Property {
    type EmptyProp;
    const __IRISIA_EMPTY_PROP: Self::EmptyProp;
}

pub trait PropAssign<T: Definition> {
    fn prop_assign(value: T::Value) -> Self;
}

impl<T, D> PropAssign<D> for T
where
    D: Definition<Value = T>,
{
    fn prop_assign(value: T) -> Self {
        value
    }
}
