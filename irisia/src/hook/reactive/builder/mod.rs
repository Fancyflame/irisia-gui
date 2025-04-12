use std::{
    cell::OnceCell,
    collections::VecDeque,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use super::{CallbackFnAlias, Inner, Reactive, WeakReactive};
use crate::hook::{
    signal_group::SignalGroup,
    utils::{trace_cell::TraceMut, TraceCell},
};
use callback_chain::{CallbackChain, CallbackNode};

mod callback_chain;

pub struct ReactiveBuilder<T, C> {
    _value: PhantomData<T>,
    callbacks: C,
}

impl<T> Reactive<T> {
    pub fn builder() -> ReactiveBuilder<T, ()> {
        ReactiveBuilder {
            _value: PhantomData,
            callbacks: (),
        }
    }
}

impl<T, C> ReactiveBuilder<T, C>
where
    T: 'static,
{
    pub fn dep<F, D>(
        self,
        mut callback: F,
        deps: D,
    ) -> ReactiveBuilder<T, CallbackNode<impl CallbackFnAlias<T, D>, D, C>>
    where
        F: FnMut(&mut T, D::Data<'_>) + 'static,
        D: SignalGroup + 'static,
    {
        self.dep2(move |mut val, data| callback(&mut val, data), deps)
    }

    pub fn dep2<F, D>(self, callback: F, deps: D) -> ReactiveBuilder<T, CallbackNode<F, D, C>>
    where
        F: CallbackFnAlias<T, D>,
        D: SignalGroup + 'static,
    {
        ReactiveBuilder {
            _value: self._value,
            callbacks: CallbackNode {
                deps,
                callback: callback.into(),
                next: self.callbacks,
            },
        }
    }

    pub fn build_cyclic<F>(self, create_init_state: F) -> Reactive<T>
    where
        F: FnOnce(&WeakReactive<T>) -> T,
        C: CallbackChain<T> + 'static,
    {
        let inner = Rc::new_cyclic(move |weak| {
            self.callbacks.listen(weak.clone(), |inner| {
                inner
                    .callback_chain_storage
                    .downcast_ref()
                    .expect("callback chain and signal mismatched")
            });
            Inner {
                value: TraceCell::new(create_init_state(&WeakReactive(weak.clone()))),
                delay_callbacks: VecDeque::new().into(),
                callback_chain_storage: Box::new(self.callbacks),
            }
        });

        Reactive { inner }
    }

    pub fn build(self, init_state: T) -> Reactive<T>
    where
        C: CallbackChain<T> + 'static,
    {
        self.build_cyclic(|_| init_state)
    }
}

pub enum ReactiveRef<'a, T> {
    Real(RealRef<'a, T>),
    Raw(&'a mut T),
}

impl<T> Deref for ReactiveRef<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Real(r) => &r,
            Self::Raw(r) => r,
        }
    }
}

impl<T> DerefMut for ReactiveRef<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Real(r) => &mut *r,
            Self::Raw(r) => r,
        }
    }
}

impl<T> ReactiveRef<'_, T> {
    /// Temporarily releases the internal reference to allow this `Reactive` to be borrowed by other functions.
    /// The internal reference will be reacquired upon the next dereference attempt.
    pub fn drop_borrow(this: &mut Self) {
        if let Self::Real(r) = this {
            r.r.take();
        }
    }
}

pub struct RealRef<'a, T> {
    pub(crate) trace_cell: &'a TraceCell<T>,
    pub(crate) r: OnceCell<TraceMut<'a, T>>,
}

impl<'a, T> RealRef<'a, T> {
    pub(crate) fn new(cell: &'a TraceCell<T>, trace_mut: TraceMut<'a, T>) -> Self {
        RealRef {
            trace_cell: cell,
            r: OnceCell::from(trace_mut),
        }
    }

    fn init_ref(&self) -> &T {
        self.r.get_or_init(|| {
            self.trace_cell
                .borrow_mut()
                .expect("cannot deref while `Reactive` is still in borrow")
        })
    }
}

impl<T> Deref for RealRef<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.init_ref()
    }
}

impl<T> DerefMut for RealRef<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.init_ref();
        self.r.get_mut().unwrap()
    }
}
