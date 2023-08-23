use crate::{
    dom::update::EMUpdateContent, structure::MapVisit, update_with::SpecificUpdate, UpdateWith,
};

use super::RenderMultiple;

type MapOutput<'a, T> = <T as MapVisit<EMUpdateContent<'a>>>::Output;

pub trait ChildrenNodes<'a>
where
    Self: MapVisit<EMUpdateContent<'a>, Output = Self::AliasMapOutput>,
{
    type AliasMapOutput: SpecificUpdate<UpdateTo = Self::AliasUpdateTo>;
    type AliasUpdateTo: RenderMultiple + UpdateWith<MapOutput<'a, Self>> + 'static;
}

impl<'a, T> ChildrenNodes<'a> for T
where
    T: MapVisit<EMUpdateContent<'a>>,
    T::Output: SpecificUpdate,
    <MapOutput<'a, T> as SpecificUpdate>::UpdateTo:
        RenderMultiple + UpdateWith<MapOutput<'a, T>> + 'static,
{
    type AliasMapOutput = T::Output;
    type AliasUpdateTo = <Self::AliasMapOutput as SpecificUpdate>::UpdateTo;
}

pub struct AsChildrenWrapper<T>(T);

impl<T>