use super::{StructureUpdateTo, VisitBy};
use crate::{
    dep_watch::{bitset::U32Array, inferer::BitsetInc},
    Result,
};

impl VisitBy for () {
    type AddUpdatePoints<Base: BitsetInc> = Base;
    const UPDATE_POINTS: u32 = 0;

    fn visit_by<V>(&self, _: &mut V) -> Result<()>
    where
        V: super::VisitOn,
    {
        Ok(())
    }

    fn len(&self) -> usize {
        0
    }

    fn is_empty(&self) -> bool {
        true
    }
}

impl<A: U32Array> StructureUpdateTo<A> for () {
    type Target = ();

    fn create(self, _: super::Updating<A>) -> Self::Target {}
    fn update(self, _: &mut Self::Target, _: super::Updating<A>) {}
}
