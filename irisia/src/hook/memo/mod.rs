use std::{
    ops::{Deref, DerefMut},
    rc::{Rc, Weak},
};

use super::{
    listener::CallbackAction,
    provider_group::ProviderGroup,
    trace_cell::TraceCell,
    utils::{DirtyCounter, ListenerList},
    Listener, Provider, Ref,
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
        Self::new_customized(
            logic(deps.read_many()),
            move |mut setter, data| {
                let new_value = logic(data);
                if *setter != new_value {
                    *setter = new_value;
                }
            },
            deps,
        )
    }

    pub fn new_customized<F, D>(init_state: T, logic: F, deps: D) -> Self
    where
        F: Fn(Setter<'_, T>, D::Data<'_>) + 'static,
        D: ProviderGroup + 'static,
    {
        let inner = Rc::new_cyclic(|weak: &Weak<Inner<T, _>>| {
            let listener = Listener::new(weak.clone(), Inner::callback);
            deps.dependent_many(listener);

            Inner {
                listener_list: ListenerList::new(),
                dirty_counter: DirtyCounter::new(),
                value: TraceCell::new(init_state),
                update_fn: move |setter: Setter<T>| logic(setter, deps.read_many()),
            }
        });

        Self { inner }
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

struct Inner<T, F> {
    listener_list: ListenerList,
    dirty_counter: DirtyCounter,
    value: TraceCell<T>,
    update_fn: F,
}

impl<T, F> Inner<T, F>
where
    F: Fn(Setter<T>),
{
    fn callback(&self, action: CallbackAction) {
        let Some(action) = self.dirty_counter.push(action) else {
            return;
        };

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
        Ref::CellRef(self.value.borrow().unwrap())
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
