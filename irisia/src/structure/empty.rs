use super::VisitBy;
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
