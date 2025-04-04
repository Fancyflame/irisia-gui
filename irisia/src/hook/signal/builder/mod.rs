use std::{
    ops::{Deref, DerefMut},
    rc::Rc,
};

use super::{Inner, Signal, WriteSignal};
use crate::hook::{
    signal_group::SignalGroup,
    utils::{DirtyCount, ListenerList, TraceCell},
};
use callback_chain::{CallbackChain, CallbackNode};

mod callback_chain;

pub struct SignalBuilder<T, C, W> {
    pub(super) value: T,
    pub(super) callbacks: C,
    pub(super) writable: W,
}

impl<T, C, W> SignalBuilder<T, C, W>
where
    T: 'static,
{
    pub fn dep<F, D>(self, callback: F, deps: D) -> SignalBuilder<T, CallbackNode<F, D, C>, W>
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
            writable: self.writable,
        }
    }

    pub fn writable(self) -> SignalBuilder<T, C, WriteMode> {
        SignalBuilder {
            value: self.value,
            callbacks: self.callbacks,
            writable: WriteMode,
        }
    }

    pub fn dep_call<F, D>(
        mut self,
        callback: F,
        deps: D,
        enable: bool,
    ) -> SignalBuilder<T, CallbackNode<F, D, C>, W>
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

impl<T, C> SignalBuilder<T, C, ()>
where
    T: 'static,
{
    pub fn build(self) -> Signal<T>
    where
        C: CallbackChain<T> + 'static,
    {
        self.writable().build().0
    }
}

impl<T, C> SignalBuilder<T, C, WriteMode>
where
    T: 'static,
{
    pub fn build(self) -> WriteSignal<T>
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

        WriteSignal(Signal { inner })
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

pub struct WriteMode;
