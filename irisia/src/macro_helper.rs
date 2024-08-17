use crate::element::{Component, ComponentTemplate};

use crate::{data_flow::Readable, ElementInterfaces};

pub type ElementPropsAlias<'a, T> = <T as ElementInterfaces>::Props<'a>;

pub trait ElementTypeHelper<_T> {
    type Target: ElementInterfaces;
}

pub struct UseSelf;

impl<T> ElementTypeHelper<UseSelf> for T
where
    T: ElementInterfaces,
{
    type Target = Self;
}

pub struct UseComponent;

impl<T> ElementTypeHelper<UseComponent> for T
where
    T: ComponentTemplate,
{
    type Target = Component<T>;
}

/// To prevent `&T` from being cloned
pub trait CloneHelper {
    fn __irisia_clone_wire(&self) -> Self;
}

impl<T> CloneHelper for T
where
    T: Readable + Clone,
{
    fn __irisia_clone_wire(&self) -> Self {
        self.clone()
    }
}
