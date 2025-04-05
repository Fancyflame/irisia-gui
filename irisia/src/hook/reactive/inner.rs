use std::{any::Any, cell::RefCell, collections::VecDeque};

use crate::hook::utils::TraceCell;

type DelayCallback<T> = Box<dyn Fn(&Inner<T>, &mut T)>;

pub struct Inner<T: ?Sized> {
    pub(super) callback_chain_storage: Box<dyn Any>,
    pub(super) delay_callbacks: RefCell<VecDeque<DelayCallback<T>>>,
    pub(super) value: TraceCell<T>,
}

impl<T: ?Sized> Inner<T> {
    pub(super) fn recall_delayed_callback(&self, value: &mut T) {
        while let Some(delayed) = self.delay_callbacks.borrow_mut().pop_front() {
            delayed(self, value);
        }
    }
}
