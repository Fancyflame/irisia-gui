use std::{
    cell::RefCell,
    panic::{catch_unwind, resume_unwind, AssertUnwindSafe},
};

thread_local! {
    static UPDATE_POINT_STACK: RefCell<Vec<usize>> = RefCell::new(Vec::new());
}

pub(crate) fn with_caller<F, R>(id: usize, f: F) -> R
where
    F: FnOnce() -> R,
{
    UPDATE_POINT_STACK.with_borrow_mut(|stack| {
        stack.push(id);
    });

    let result = catch_unwind(AssertUnwindSafe(f));

    UPDATE_POINT_STACK.with_borrow_mut(|stack| {
        stack.pop();
    });

    match result {
        Ok(r) => r,
        Err(e) => resume_unwind(e),
    }
}

pub(crate) fn get_caller() -> Option<usize> {
    UPDATE_POINT_STACK.with_borrow(|stack| stack.last().copied())
}
