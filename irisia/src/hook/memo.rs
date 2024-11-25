use std::{
    ops::{Deref, DerefMut},
    rc::{Rc, Weak},
};

use super::{
    listener::CallbackAction,
    provider_group::ProviderGroup,
    utils::{ListenerList, TraceCell},
    Listener, Provider, ProviderObject, Ref, ToProviderObject,
};

pub struct Memo<T> {
    inner: Rc<dyn Provider<Data = T>>,
}

impl<T: 'static> Memo<T> {
    pub fn new<F, D>(logic: F, deps: D) -> Self
    where
        T: Eq,
        F: Fn(D::Data<'_>) -> T + 'static,
        D: ProviderGroup + 'static,
    {
        let inner = Rc::new_cyclic(|weak| {
            let listener = Listener::new(weak.clone(), Inner::callback);
            deps.dependent_many(listener);

            let generator = move || logic(D::deref_wrapper(&deps.read_many()));

            Inner {
                listener_list: ListenerList::new(),
                value: TraceCell::new(generator()),
                update_fn: move |mut setter: Setter<T>| {
                    let new_value = generator();
                    if *setter != new_value {
                        *setter = new_value;
                    }
                },
            }
        });
        Self { inner }
    }

    pub fn new_customized<F, D>(mut init_state: T, logic: F, deps: D) -> Self
    where
        F: Fn(Setter<'_, T>, D::Data<'_>) + 'static,
        D: ProviderGroup + 'static,
    {
        let inner = Rc::new_cyclic(|weak: &Weak<Inner<T, _>>| {
            let listener = Listener::new(weak.clone(), Inner::callback);
            deps.dependent_many(listener);
            let update_fn =
                move |setter: Setter<T>| logic(setter, D::deref_wrapper(&deps.read_many()));
            update_fn(Setter {
                r: &mut init_state,
                mutated: &mut true,
            });

            Inner {
                listener_list: ListenerList::new(),
                value: TraceCell::new(init_state),
                update_fn,
            }
        });

        Self { inner }
    }
}

struct Inner<T, F> {
    listener_list: ListenerList,
    value: TraceCell<T>,
    update_fn: F,
}

impl<T, F> Inner<T, F>
where
    F: Fn(Setter<T>),
{
    fn callback(&self, action: CallbackAction) {
        match action {
            CallbackAction::RegisterDirty | CallbackAction::ClearDirty => {
                self.listener_list.callback_all(action);
                return;
            }
            CallbackAction::Update => {}
        }

        let mut value = self.value.borrow_mut().unwrap();
        let mut mutated = false;
        (self.update_fn)(Setter {
            r: &mut *value,
            mutated: &mut mutated,
        });
        drop(value);

        self.listener_list.callback_all(if mutated {
            CallbackAction::Update
        } else {
            CallbackAction::ClearDirty
        });
    }
}

impl<T, F> Provider for Inner<T, F> {
    type Data = T;

    fn read(&self) -> super::Ref<Self::Data> {
        Ref::TraceRef(self.value.borrow().unwrap())
    }

    fn dependent(&self, listener: Listener) {
        self.listener_list.add_listener(listener);
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

impl<T> ToProviderObject for Memo<T> {
    type Data = T;
    fn to_object(&self) -> ProviderObject<Self::Data> {
        ProviderObject(self.inner.clone())
    }
}

impl<T> Provider for Memo<T> {
    type Data = T;
    fn read(&self) -> Ref<Self::Data> {
        self.inner.read()
    }
    fn dependent(&self, listener: Listener) {
        self.inner.dependent(listener);
    }
}

impl<T> Deref for Memo<T> {
    type Target = dyn Provider<Data = T>;
    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

impl<T> Clone for Memo<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
