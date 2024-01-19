pub use select_chain::SelectBody;

use self::select_chain::SelectVisitBy;

use super::{Slot, VisitBy, VisitOn};

mod select_chain;

pub struct SelectHead<T> {
    pub branch_index: usize,
    pub branches: T,
}

impl<T: SelectVisitBy> SelectHead<T> {
    pub fn extend_branch<U: VisitBy, Slt>(
        self,
        slot: Slot<Slt>,
    ) -> SelectHead<T::ExtendNode<U, Slt>> {
        SelectHead {
            branch_index: self.branch_index,
            branches: self.branches.extend(slot),
        }
    }
}

impl<T> VisitBy for SelectHead<T>
where
    T: SelectVisitBy,
{
    fn visit_by<V: VisitOn>(&self, visitor: &mut V) -> crate::Result<()> {
        self.branches.visit(self.branch_index, visitor)
    }

    fn len(&self) -> usize {
        self.branches.len(self.branch_index)
    }
}
