use super::{PropCast, PropUpdate, Property, SignalPropAgent};
use crate::{
    Signal,
    model::component::{definition::UsingDefault, property::PropertyAgent},
};

type Agent<T> = SignalPropAgent<Option<Signal<T>>>;

impl<T: ?Sized> Property for Option<Signal<T>> {
    type Agent = Agent<T>;
    fn __irisia_prop_agent<'a>() -> &'a Self::Agent
    where
        Self::Agent: 'a,
    {
        &Agent::GET
    }
}

impl<T> PropertyAgent for Agent<T>
where
    T: ?Sized,
{
    type CastTarget = Option<Signal<T>>;

    type Empty = UsingDefault<T>;
    fn get_empty(&self) -> Self::Empty {
        UsingDefault::GET
    }
}
