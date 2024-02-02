use super::{StructureUpdateTo, VisitBy};
use crate::Result;

impl VisitBy for () {
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

impl<const WD: usize> StructureUpdateTo<WD> for () {
    type Target = ();
    const UPDATE_POINTS: u32 = 0;

    fn create(self, _: super::Updating<WD>) -> Self::Target {}
    fn update(self, _: &mut Self::Target, _: super::Updating<WD>) {}
}
