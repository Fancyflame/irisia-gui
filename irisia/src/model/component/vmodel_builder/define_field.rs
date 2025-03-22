use crate::model::tools::{caller_stack, DirtyPoints};

use super::VModelBuilderNode;

pub struct DefineField<F>(pub(super) F);

impl<T, F> VModelBuilderNode<'_, T> for DefineField<F>
where
    F: FnOnce(&mut T),
{
    const EXECUTE_POINTS: usize = 1;

    fn create_build(self, src: &mut T, exec_point_offset: usize) {
        caller_stack::with_caller(exec_point_offset, || (self.0)(src));
    }

    fn update_build(self, src: &mut T, mut dp: DirtyPoints) {
        if dp.check_range(1) {
            caller_stack::with_caller(dp.offset(), || (self.0)(src));
        }
    }
}
