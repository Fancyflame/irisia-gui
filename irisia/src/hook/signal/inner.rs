use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use smallvec::SmallVec;

use crate::hook::{
    Listener,
    utils::{CallbackAction, DirtyCount, ListenerList, TraceCell, trace_cell::TraceRef},
};

pub struct StrongListenerList(pub(super) SmallVec<[Rc<dyn Dependency>; 1]>);

pub struct Inner<T: ?Sized> {
    pub(super) listeners: ListenerList,
    pub(super) global_dirty_count: DirtyCount,
    pub(super) dep_list: StrongListenerList,
    pub(super) delay_update_indexes: RefCell<VecDeque<usize>>,
    pub(super) value: TraceCell<T>,
}

impl<T> Inner<T>
where
    T: ?Sized,
{
    pub(super) fn read(&self) -> TraceRef<'_, T> {
        self.value.borrow().unwrap()
    }

    pub(super) fn dependent(&self, listener: Listener) {
        self.listeners.add_listener(listener);
    }

    pub(super) fn push_action(&self, action: CallbackAction) {
        if let Some(action) = self.global_dirty_count.push(action) {
            self.listeners.callback_all(action);
        };
    }
}

pub(super) trait Dependency {
    fn manual_update(&self);
}
