use std::{
    ops::{Deref, DerefMut},
    rc::Rc,
};

use super::{Inner, Signal, inner::StrongListenerList};
use crate::hook::{
    signal_group::SignalGroup,
    utils::{DirtyCount, ListenerList, TraceCell},
};
use callback_chain::{CallbackChain, CallbackNode};
use smallvec::SmallVec;

mod callback_chain;

pub struct SignalBuilder<T, C> {
    pub(super) value: T,
    pub(super) callbacks: C,
}

impl<T, C> SignalBuilder<T, C>
where
    T: 'static,
{
    pub fn dep<F, D>(self, callback: F, deps: D) -> SignalBuilder<T, CallbackNode<F, D, C>>
    where
        F: Fn(Setter<T>, D::Data<'_>) + 'static,
        D: SignalGroup + 'static,
    {
        SignalBuilder {
            value: self.value,
            callbacks: CallbackNode {
                deps,
                callback,
                next: self.callbacks,
            },
        }
    }

    pub fn dep_call<F, D>(
        mut self,
        callback: F,
        deps: D,
        enable: bool,
    ) -> SignalBuilder<T, CallbackNode<F, D, C>>
    where
        F: Fn(Setter<T>, D::Data<'_>) + 'static,
        D: SignalGroup + 'static,
    {
        if enable {
            callback(
                Setter {
                    r: &mut self.value,
                    mutated: &mut true,
                },
                D::deref_wrapper(&deps.read_many()),
            );
        }

        self.dep(callback, deps)
    }
}

impl<T, C> SignalBuilder<T, C>
where
    T: 'static,
{
    pub fn build(self) -> Signal<T>
    where
        C: CallbackChain<T> + 'static,
    {
        let inner = Rc::new_cyclic(|weak| {
            let mut store_list = StrongListenerList(SmallVec::new());
            self.callbacks.listen(weak.clone(), &mut store_list);
            Inner {
                value: TraceCell::new(self.value),
                global_dirty_count: DirtyCount::new(),
                _store_list: store_list,
                listeners: ListenerList::new(),
            }
        });

        Signal { inner }
    }
}

pub struct Setter<'a, T: ?Sized> {
    r: &'a mut T,
    mutated: &'a mut bool,
}

impl<T: ?Sized> Deref for Setter<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.r
    }
}

impl<T: ?Sized> DerefMut for Setter<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        *self.mutated = true;
        self.r
    }
}

impl<'a, T> Setter<'a, T> {
    pub fn new(r: &'a mut T, mutated: &'a mut bool) -> Self {
        Self { r, mutated }
    }
}
