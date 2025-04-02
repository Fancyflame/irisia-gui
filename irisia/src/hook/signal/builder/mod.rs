use std::{
    ops::{Deref, DerefMut},
    rc::Rc,
};

use super::{Inner, Signal};
use crate::hook::{
    provider_group::ProviderGroup,
    utils::{DirtyCount, ListenerList, TraceCell},
};
use callback_chain::{CallbackChain, CallbackNode};

mod callback_chain;

pub struct SignalBuilder<T, C> {
    pub(super) value: T,
    pub(super) callbacks: C,
}

impl<T: 'static, C> SignalBuilder<T, C> {
    pub fn dep<F, D>(self, callback: F, deps: D) -> SignalBuilder<T, CallbackNode<F, D, C>>
    where
        F: Fn(Setter<T>, D::Data<'_>) + 'static,
        D: ProviderGroup + 'static,
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
        D: ProviderGroup + 'static,
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

    pub fn build(self) -> Signal<T>
    where
        C: CallbackChain<T> + 'static,
    {
        let inner = Rc::new_cyclic(|weak| {
            self.callbacks.listen(weak.clone(), |inner| {
                inner
                    .callback_chain_storage
                    .downcast_ref()
                    .expect("callback chain and signal mismatched")
            });
            Inner {
                value: TraceCell::new(self.value),
                global_dirty_count: DirtyCount::new(),
                callback_chain_storage: Box::new(self.callbacks),
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
