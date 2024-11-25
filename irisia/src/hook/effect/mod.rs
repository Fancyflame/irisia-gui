use std::{
    cell::Cell,
    rc::{Rc, Weak},
};

use state::EffectState;

use super::{
    listener::CallbackAction,
    provider_group::ProviderGroup,
    utils::{ListenerList, TraceCell},
    Listener, Provider, ProviderObject, Ref, ToProviderObject,
};

mod state;

pub struct Effect<T> {
    inner: ProviderObject<T>,
}

impl<T: 'static> Effect<T> {
    pub fn new<F, Fd, D>(init_state: T, make_effect: F, deps: D) -> Self
    where
        D: ProviderGroup + 'static,
        F: Fn(EffectState<T>, D::Data<'_>) -> Fd + 'static,
        Fd: FnOnce() + 'static,
    {
        let inner = Rc::new_cyclic(|weak: &Weak<Inner<T, F, Fd, D>>| {
            let listener = Listener::new(weak.clone(), Inner::callback);
            deps.dependent_many(listener);

            Inner {
                state: TraceCell::new(init_state),
                effect_state: EffectState::new(weak.clone()),
                listener_list: ListenerList::new(),
                fn_drop: Cell::new(None),
                make_effect,
                deps,
            }
        });
        inner.callback(CallbackAction::Update);

        Self {
            inner: ProviderObject(inner),
        }
    }
}

struct Inner<T, F, Fd, D>
where
    Fd: FnOnce(),
{
    state: TraceCell<T>,
    effect_state: EffectState<T>,
    listener_list: ListenerList,
    make_effect: F,
    fn_drop: Cell<Option<Fd>>,
    deps: D,
}

impl<T, F, Fd, D> Inner<T, F, Fd, D>
where
    D: ProviderGroup,
    F: Fn(EffectState<T>, D::Data<'_>) -> Fd,
    Fd: FnOnce(),
{
    fn callback(&self, action: CallbackAction) {
        if !action.is_update() {
            return;
        }

        if let Some(fn_drop) = self.fn_drop.take() {
            fn_drop();
        }

        self.fn_drop.set(Some((self.make_effect)(
            self.effect_state.clone(),
            D::deref_wrapper(&self.deps.read_many()),
        )));
    }
}

impl<T, F, Fd: FnOnce(), D> Provider for Inner<T, F, Fd, D> {
    type Data = T;
    fn dependent(&self, listener: Listener) {
        self.listener_list.add_listener(listener);
    }
    fn read(&self) -> Ref<Self::Data> {
        Ref::TraceRef(
            self.state
                .borrow()
                .expect("cannot read `Effect` when updating"),
        )
    }
}

impl<T, F, Fd, D> Drop for Inner<T, F, Fd, D>
where
    Fd: FnOnce(),
{
    fn drop(&mut self) {
        if let Some(fn_drop) = self.fn_drop.take() {
            fn_drop();
        }
    }
}

impl<T> Provider for Effect<T> {
    type Data = T;
    fn dependent(&self, listener: Listener) {
        self.inner.dependent(listener);
    }
    fn read(&self) -> Ref<Self::Data> {
        self.inner.read()
    }
}

impl<T> ToProviderObject for Effect<T> {
    type Data = T;
    fn to_object(&self) -> ProviderObject<Self::Data> {
        self.inner.clone()
    }
}

impl<T> Clone for Effect<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

/*trait InnerTrait<T> {
    fn as_inner(&self) -> Inner<T, impl F>
}*/
