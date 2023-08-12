use super::{MapVisit, UpdateWith, Visit, VisitLen, VisitMut};
use crate::{update_with::SpecificUpdate, Result};

impl<V> MapVisit<V> for () {
    type Output = ();
    fn map(self, _: &V) {}
}

impl VisitLen for () {
    fn len(&self) -> usize {
        0
    }
}

impl<V> Visit<V> for () {
    fn visit(&self, _: &mut V) -> Result<()> {
        Ok(())
    }
}

impl<V> VisitMut<V> for () {
    fn visit_mut(&mut self, _: &mut V) -> Result<()> {
        Ok(())
    }
}

impl UpdateWith<()> for () {
    fn create_with(_: ()) {}

    fn update_with(&mut self, _: (), equality_matters: bool) -> bool {
        equality_matters
    }
}

impl SpecificUpdate for () {
    type UpdateTo = ();
}
