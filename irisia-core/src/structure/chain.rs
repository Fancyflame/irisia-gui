use crate::update_with::SpecificUpdate;

use super::MapVisit;
use super::{ControlFlow, UpdateWith, VisitLen, VisitMut};

use super::Visit;

pub struct Chain<A, B> {
    former: A,
    latter: B,
}

impl<A, B, V> MapVisit<V> for Chain<A, B>
where
    A: MapVisit<V>,
    B: MapVisit<V>,
{
    type Output = Chain<A::Output, B::Output>;
    fn map(self, visitor: &V) -> Self::Output {
        Chain {
            former: self.former.map(visitor),
            latter: self.latter.map(visitor),
        }
    }
}

impl<A, B> VisitLen for Chain<A, B>
where
    A: VisitLen,
    B: VisitLen,
{
    fn len(&self) -> usize {
        self.former.len() + self.latter.len()
    }
}

impl<A, B, V> Visit<V> for Chain<A, B>
where
    A: Visit<V>,
    B: Visit<V>,
{
    fn visit_with_control_flow(&self, visitor: &mut V, control: &mut ControlFlow) {
        self.former.visit_with_control_flow(visitor, control);
        if !control.should_exit() {
            self.latter.visit_with_control_flow(visitor, control);
        }
    }
}

impl<A, B, V> VisitMut<V> for Chain<A, B>
where
    A: VisitMut<V>,
    B: VisitMut<V>,
{
    fn visit_mut_with_control_flow(&mut self, visitor: &mut V, control: &mut ControlFlow) {
        self.former.visit_mut_with_control_flow(visitor, control);
        if !control.should_exit() {
            self.latter.visit_mut_with_control_flow(visitor, control);
        }
    }
}

impl<A, B, X, Y> UpdateWith<Chain<X, Y>> for Chain<A, B>
where
    A: UpdateWith<X>,
    B: UpdateWith<Y>,
{
    fn create_with(updater: Chain<X, Y>) -> Self {
        Self {
            former: A::create_with(updater.former),
            latter: B::create_with(updater.latter),
        }
    }

    fn update_with(&mut self, updater: Chain<X, Y>, mut equality_matters: bool) -> bool {
        equality_matters &= self.former.update_with(updater.former, equality_matters);
        equality_matters &= self.latter.update_with(updater.latter, equality_matters);
        equality_matters
    }
}

impl<A, B> SpecificUpdate for Chain<A, B>
where
    A: SpecificUpdate,
    B: SpecificUpdate,
    A::UpdateTo: UpdateWith<A>,
    B::UpdateTo: UpdateWith<B>,
{
    type UpdateTo = Chain<A::UpdateTo, B::UpdateTo>;
}
