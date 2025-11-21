use std::rc::{Rc, Weak};

use crate::hook::{
    Listener,
    listener::{ListenerCallback, ListenerInner},
    signal::inner::{Dependency, StrongListenerList},
    signal_group::SignalGroup,
    utils::{CallbackAction, trace_cell::TraceMut},
};

use super::{Inner, Setter};

pub trait BuilderDepChain<T> {
    fn listen(self, src: Weak<Inner<T>>, store_list: &mut StrongListenerList);
}

impl<T> BuilderDepChain<T> for () {
    fn listen(self, _: Weak<Inner<T>>, _: &mut StrongListenerList) {}
}

pub struct BuilderDepNode<F, D, Next> {
    pub(super) deps: D,
    pub(super) callback: F,
    pub(super) next: Next,
}

impl<T, F, D, Next> BuilderDepChain<T> for BuilderDepNode<F, D, Next>
where
    T: 'static,
    D: SignalGroup + 'static,
    F: Fn(Setter<T>, D::Data<'_>) + 'static,
    Next: BuilderDepChain<T>,
{
    fn listen(self, weak_src: Weak<Inner<T>>, store_list: &mut StrongListenerList) {
        let strong_listener = Listener::new(DepInner {
            dep_list_position: store_list.0.len(),
            src: weak_src.clone(),
            callback: self.callback,
            deps: self.deps,
        });

        strong_listener
            .callback
            .deps
            .dependent_many(strong_listener.to_listener());

        store_list.0.push(strong_listener);
        self.next.listen(weak_src, store_list);
    }
}

struct DepInner<T, F, D> {
    dep_list_position: usize,
    src: Weak<Inner<T>>,
    callback: F,
    deps: D,
}

impl<T, F, D> DepInner<T, F, D>
where
    T: 'static,
    F: Fn(Setter<T>, D::Data<'_>) + 'static,
    D: SignalGroup + 'static,
{
    fn call_update(&self, src: &Rc<Inner<T>>, mut value_mut: TraceMut<'_, T>) {
        let mut mutated = false;

        (self.callback)(
            Setter::new(&mut value_mut, &mut mutated),
            D::deref_wrapper(&self.deps.read_many()),
        );

        // important: must drop `value_mut` before call listeners
        drop(value_mut);

        src.push_action(if mutated {
            CallbackAction::Update
        } else {
            CallbackAction::ClearDirty
        });
    }
}

impl<T, F, D> ListenerCallback for DepInner<T, F, D>
where
    T: 'static,
    F: Fn(Setter<T>, D::Data<'_>) + 'static,
    D: SignalGroup + 'static,
{
    fn call(&self, action: CallbackAction) -> bool {
        let src = self.src.upgrade().unwrap();

        if !action.is_update() {
            src.push_action(action);
            return true;
        }

        match src.value.try_borrow_mut() {
            Some(value) => self.call_update(&src, value),
            None => src
                .delay_update_indexes
                .try_borrow_mut()
                .unwrap_or_else(|_| {
                    panic!(
                        "Cannot modify dependencies that update a Signal while \
                        that Signal is being updated, as this would cause an infinite loop"
                    )
                })
                .push_back(self.dep_list_position),
        }

        true
    }
}

impl<T, F, D> Dependency for ListenerInner<DepInner<T, F, D>>
where
    T: 'static,
    F: Fn(Setter<T>, D::Data<'_>) + 'static,
    D: SignalGroup + 'static,
{
    fn manual_update(&self) {
        let this = &self.callback;

        let src = this.src.upgrade().unwrap();
        let value = src
            .value
            .borrow_mut()
            .unwrap_or_else(|_| unreachable!("manual update cannot be fail"));
        this.call_update(&src, value);
    }
}
