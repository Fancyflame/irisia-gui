use std::{collections::VecDeque, rc::Rc};

use super::{Inner, Reactive};
use crate::hook::{signal_group::SignalGroup, utils::TraceCell};
use callback_chain::{CallbackChain, CallbackNode};

mod callback_chain;

pub struct ReactiveBuilder<T, C> {
    value: T,
    callbacks: C,
}

impl<T> Reactive<T> {
    pub fn builder(value: T) -> ReactiveBuilder<T, ()> {
        ReactiveBuilder {
            value,
            callbacks: (),
        }
    }
}

impl<T, C> ReactiveBuilder<T, C>
where
    T: 'static,
{
    pub fn dep<F, D>(self, callback: F, deps: D) -> ReactiveBuilder<T, CallbackNode<F, D, C>>
    where
        F: FnMut(&mut T, D::Data<'_>) + 'static,
        D: SignalGroup + 'static,
    {
        ReactiveBuilder {
            value: self.value,
            callbacks: CallbackNode {
                deps,
                callback: callback.into(),
                next: self.callbacks,
            },
        }
    }

    pub fn dep_call<F, D>(
        mut self,
        mut callback: F,
        deps: D,
        enable: bool,
    ) -> ReactiveBuilder<T, CallbackNode<F, D, C>>
    where
        F: FnMut(&mut T, D::Data<'_>) + 'static,
        D: SignalGroup + 'static,
    {
        if enable {
            callback(&mut self.value, D::deref_wrapper(&deps.read_many()));
        }

        self.dep(callback, deps)
    }

    pub fn build(self) -> Reactive<T>
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
                delay_callbacks: VecDeque::new().into(),
                callback_chain_storage: Box::new(self.callbacks),
            }
        });

        Reactive { inner }
    }
}
