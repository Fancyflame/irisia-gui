use std::{any::Any, cell::RefCell, collections::VecDeque};

use crate::hook::utils::TraceCell;

use super::{builder::ReactiveRef, RealRef};

type DelayCallback<T> = Box<dyn FnOnce(&Inner<T>, ReactiveRef<T>)>;

pub struct Inner<T> {
    pub(super) callback_chain_storage: Box<dyn Any>,
    // TODO: 这个调用队列可以优化掉Box吗？
    pub(super) delay_callbacks: RefCell<VecDeque<DelayCallback<T>>>,
    pub(super) value: TraceCell<T>,
}

impl<T> Inner<T> {
    pub(super) fn recall_delayed_callback(&self) {
        while let Some(delayed) = self.delay_callbacks.borrow_mut().pop_front() {
            delayed(
                self,
                ReactiveRef::Real(RealRef::new(&self.value, self.value.borrow_mut().unwrap())),
            );
        }
    }
}
