pub use select_chain::SelectBody;

use self::select_chain::SelectVisitBy;

use super::{VisitBy, VisitOn};

mod select_chain;

pub struct SelectHead<T> {
    pub branch_index: usize,
    pub branch: T,
}

impl<T: SelectVisitBy> SelectHead<T> {
    pub fn extend_branch<U: VisitBy>(self) -> SelectHead<T::ExtendNode<U>> {
        SelectHead {
            branch_index: self.branch_index,
            branch: self.branch.extend(),
        }
    }
}

impl<T> VisitBy for SelectHead<T>
where
    T: SelectVisitBy,
{
    fn visit_by<V: VisitOn>(&self, visitor: &mut V) -> crate::Result<()> {
        self.branch.visit(self.branch_index, visitor)
    }

    fn len(&self) -> usize {
        self.branch.len(self.branch_index)
    }
}
