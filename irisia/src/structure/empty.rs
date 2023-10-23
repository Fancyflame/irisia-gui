use super::{VisitBy, VisitLen, VisitMutBy};
use crate::Result;

impl VisitLen for () {
    fn len(&self) -> usize {
        0
    }
}

impl<V> VisitBy<V> for () {
    fn visit(&self, _: &mut V) -> Result<()> {
        Ok(())
    }
}

impl<V> VisitMutBy<V> for () {
    fn visit_mut(&mut self, _: &mut V) -> Result<()> {
        Ok(())
    }
}
