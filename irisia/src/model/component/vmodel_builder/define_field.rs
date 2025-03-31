use crate::model::tools::{caller_stack, DirtyPoints};

use super::VModelBuilderNode;

pub struct DefineField<F>(pub(super) F);

impl<T, F> VModelBuilderNode<'_, T> for DefineField<F>
where
    F: FnOnce(&mut T),
{
    const EXECUTE_POINTS: usize = 1;

    fn create_build(self, src: &mut T, dp: &mut DirtyPoints) {
        caller_stack::with_caller(dp.offset(), || (self.0)(src));
        dp.consume(1);
    }

    fn update_build(self, src: &mut T, dp: &mut DirtyPoints) {
        if dp.check_range(1) {
            caller_stack::with_caller(dp.offset(), || (self.0)(src));
        }
        dp.consume(1);
    }
}
