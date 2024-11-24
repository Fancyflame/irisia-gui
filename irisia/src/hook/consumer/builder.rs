use std::{cell::RefCell, rc::Rc};

use crate::hook::provider_group::ProviderGroup;

use super::{
    callback_chain::{CallbackChain, CallbackNode},
    Consumer, Inner,
};

pub struct ConsumerBuilder<T, C> {
    pub(super) value: T,
    pub(super) callbacks: C,
}

impl<T, C> ConsumerBuilder<T, C> {
    pub fn dep<F, D>(self, callback: F, deps: D) -> ConsumerBuilder<T, CallbackNode<F, D, C>>
    where
        F: Fn(&mut T, D::Data<'_>) + 'static,
        D: ProviderGroup + 'static,
    {
        ConsumerBuilder {
            value: self.value,
            callbacks: CallbackNode {
                deps,
                callback,
                next: self.callbacks,
            },
        }
    }

    pub fn build(self) -> Consumer<T>
    where
        C: CallbackChain<Inner<T, C>> + 'static,
    {
        let inner = Rc::new_cyclic(|weak| {
            self.callbacks
                .listen(weak.clone(), |inner| &inner.callbacks);
            Inner {
                value: RefCell::new(self.value),
                callbacks: self.callbacks,
            }
        });
        Consumer { inner }
    }
}
