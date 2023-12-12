use super::{UpdateWith, VisitBy, VisitOn};
use crate::{
    dom::DropProtection, update_with::SpecificUpdate, ChildNodes, Element, Result, StyleGroup,
};

pub struct Once<T>(pub T);

impl<El, Sty, Slt> VisitBy for Once<DropProtection<El, Sty, Slt>>
where
    El: Element,
    Sty: StyleGroup,
    Slt: ChildNodes,
{
    fn visit_by<V>(&self, visitor: &mut V) -> Result<()>
    where
        V: VisitOn,
    {
        visitor.visit_on(&self.0)
    }

    fn len(&self) -> usize {
        1
    }

    fn is_empty(&self) -> bool {
        false
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
