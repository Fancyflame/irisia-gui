use crate::update_with::SpecificUpdate;

use super::MapVisit;
use super::{ControlFlow, UpdateWith, VisitLen, VisitMut};

use super::Visit;

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
    fn visit_with_control_flow(&self, _: &mut V, _: &mut ControlFlow) {}
}

impl<V> VisitMut<V> for () {
    fn visit_mut_with_control_flow(&mut self, _: &mut V, _: &mut ControlFlow) {}
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
