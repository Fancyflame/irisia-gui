pub use select_chain::SelectBody;

use self::select_chain::{SelectVisitBy, SelectVisitLen, SelectVisitMutBy};

use super::{VisitBy, VisitLen, VisitMutBy};

mod select_chain;

pub struct SelectHead<T> {
    pub branch_index: usize,
    pub branch: T,
}

impl<T> VisitLen for SelectHead<T>
where
    T: SelectVisitLen,
{
    fn len(&self) -> usize {
        self.branch.len(self.branch_index)
    }
}

impl<T, V> VisitBy<V> for SelectHead<T>
where
    T: SelectVisitBy<V>,
{
    fn visit(&self, visitor: &mut V) -> crate::Result<()> {
        self.branch.visit(self.branch_index, visitor)
    }
}

impl<T, V> VisitMutBy<V> for SelectHead<T>
where
    T: SelectVisitMutBy<V>,
{
    fn visit_mut(&self, visitor: &mut V) -> crate::Result<()> {
        self.branch.visit_mut(self.branch_index, visitor)
    }
}
