use super::{MapVisit, UpdateWith, Visit, VisitLen, VisitMut};
use crate::{update_with::SpecificUpdate, Result};

pub struct Chain<A, B> {
    pub former: A,
    pub latter: B,
}

impl<A, B> Chain<A, B> {
    pub fn new(former: A, latter: B) -> Self {
        Chain { former, latter }
    }
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
    fn visit(&self, visitor: &mut V) -> Result<()> {
        self.former.visit(visitor)?;
        self.latter.visit(visitor)
    }
}

impl<A, B, V> VisitMut<V> for Chain<A, B>
where
    A: VisitMut<V>,
    B: VisitMut<V>,
{
    fn visit_mut(&mut self, visitor: &mut V) -> Result<()> {
        self.former.visit_mut(visitor)?;
        self.latter.visit_mut(visitor)
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
