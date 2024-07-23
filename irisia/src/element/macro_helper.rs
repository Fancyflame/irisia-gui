use super::*;

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
