use crate::{
    Property,
    model::component::property::{PropCast, PropUpdate, PropertyAgent},
};

pub type AgentOf<T> = <T as Property>::Agent;
pub type EmptyOf<T> = <AgentOf<T> as PropertyAgent>::Empty;

pub type PropUpdateResultAny<P, Old, Updator, SourceFrom> =
    <AgentOf<P> as PropUpdate<Old, Updator, SourceFrom>>::Output;

pub type PropUpdateResult<P, Old, Updator> = PropUpdateResultAny<P, Old, Updator, AgentOf<P>>;

pub fn get_agent<'a, T>() -> &'a T::Agent
where
    T: Property + 'a,
{
    T::__irisia_prop_agent()
}

pub fn get_empty<P: Property>() -> EmptyOf<P> {
    P::__irisia_prop_agent().get_empty()
}

pub fn prop_update_extend<P, Old, Updator, SourceFrom>(
    old: Old,
    updator: Updator,
) -> PropUpdateResultAny<P, Old, Updator, SourceFrom>
where
    P: Property,
    AgentOf<P>: PropUpdate<Old, Updator, SourceFrom>,
{
    P::__irisia_prop_agent().prop_update(old, updator)
}

pub fn prop_update<P, Old, Updator>(old: Old, updator: Updator) -> PropUpdateResult<P, Old, Updator>
where
    P: Property,
    AgentOf<P>: PropUpdate<Old, Updator>,
{
    P::__irisia_prop_agent().prop_update(old, updator)
}

pub fn prop_cast<P, T>(from: T) -> P
where
    P: Property,
    AgentOf<P>: PropCast<T>,
{
    P::__irisia_prop_agent().prop_cast(from)
}
