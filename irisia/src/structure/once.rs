use super::{UpdateWith, VisitBy, VisitLen, VisitMutBy, Visitor, VisitorMut};
use crate::{update_with::SpecificUpdate, Result};

pub struct Once<T>(pub T);

impl<T> VisitLen for Once<T> {
    fn len(&self) -> usize {
        1
    }
}

impl<T, V> VisitBy<V> for Once<T>
where
    V: Visitor<T>,
{
    fn visit(&self, visitor: &mut V) -> Result<()> {
        visitor.visit(&self.0)
    }
}

impl<T, V> VisitMutBy<V> for Once<T>
where
    V: VisitorMut<T>,
{
    fn visit_mut(&mut self, visitor: &mut V) -> Result<()> {
        visitor.visit_mut(&mut self.0)
    }
}

impl<T, U> UpdateWith<Once<U>> for Once<T>
where
    T: UpdateWith<U>,
{
    fn create_with(updater: Once<U>) -> Self {
        Once(T::create_with(updater.0))
    }

    fn update_with(&mut self, updater: Once<U>, equality_matters: bool) -> bool {
        self.0.update_with(updater.0, equality_matters) && equality_matters
    }
}

impl<T> SpecificUpdate for Once<T>
where
    T: SpecificUpdate,
    T::UpdateTo: UpdateWith<T>,
{
    type UpdateTo = Once<T::UpdateTo>;
}
