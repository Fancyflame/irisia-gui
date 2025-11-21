use std::{
    fmt::{Debug, Formatter},
    marker::PhantomData,
    ops::Deref,
    rc::Rc,
};

use builder::SignalBuilder;

use super::{
    Listener,
    signal_group::SignalGroup,
    utils::{WriteGuard, trace_cell::TraceRef},
};
use inner::Inner;

pub use coerce::SignalCast;

mod builder;
mod coerce;
mod inner;

pub struct Signal<T: ?Sized> {
    inner: Rc<Inner<T>>,
}

impl<T: 'static> Signal<T> {
    pub fn state(value: T) -> WriteSignal<T> {
        WriteSignal(Self::builder().build(value))
    }

    pub fn memo<F, D>(deps: D, generator: F) -> Self
    where
        T: PartialEq<T>,
        F: Fn(D::Data<'_>) -> T + 'static,
        D: SignalGroup + 'static,
    {
        let value = generator(D::deref_wrapper(&deps.read_many()));
        Self::builder()
            .dep(
                move |mut this, data| {
                    let new_value = generator(data);
                    if *this != new_value {
                        *this = new_value;
                    }
                },
                deps,
            )
            .build(value)
    }

    pub fn memo_ncmp<F, D>(deps: D, generator: F) -> Self
    where
        F: Fn(D::Data<'_>) -> T + 'static,
        D: SignalGroup + 'static,
    {
        let value = generator(D::deref_wrapper(&deps.read_many()));
        Self::builder()
            .dep(
                move |mut this, data| {
                    *this = generator(data);
                },
                deps,
            )
            .build(value)
    }

    pub fn builder() -> SignalBuilder<T, ()> {
        SignalBuilder {
            _value: PhantomData,
            callbacks: (),
        }
    }
}

impl<T: ?Sized> Signal<T> {
    pub fn read(&self) -> SignalRef<'_, T> {
        SignalRef {
            r: self.inner.read(),
            _drop: SignalRefDrop(self),
        }
    }

    pub fn dependent(&self, l: Listener) {
        self.inner.dependent(l);
    }

    pub fn cast<U: ?Sized>(&self) -> Signal<U>
    where
        T: SignalCast<U>,
    {
        T::cast(self.clone())
    }

    pub(crate) fn addr(&self) -> *const () {
        Rc::as_ptr(&self.inner) as _
    }
}

pub struct WriteSignal<T: ?Sized>(Signal<T>);

impl<T: ?Sized> WriteSignal<T> {
    pub fn write(&self) -> WriteGuard<'_, T> {
        WriteGuard::new(
            self.0.inner.value.borrow_mut().unwrap(),
            &self.0.inner.listeners,
        )
    }

    pub fn set(&self, data: T)
    where
        T: Sized,
    {
        *self.write() = data;
    }

    pub fn read(&self) -> SignalRef<'_, T> {
        self.0.read()
    }

    pub fn to_read(&self) -> Signal<T> {
        self.0.clone()
    }

    pub(crate) fn as_read(&self) -> &Signal<T> {
        &self.0
    }
}

impl<T: ?Sized> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: ?Sized> Clone for WriteSignal<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Debug for Signal<T>
where
    T: Debug + ?Sized,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        debug_signal(f, self, false)
    }
}

impl<T> Debug for WriteSignal<T>
where
    T: Debug + ?Sized,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        debug_signal(f, &self.0, true)
    }
}

fn debug_signal<T: Debug + ?Sized>(
    f: &mut Formatter,
    signal: &Signal<T>,
    is_write_signal: bool,
) -> std::fmt::Result {
    let value: &&T = &&*signal.read();
    let struct_name = if is_write_signal {
        "WriteSignal"
    } else {
        "Signal"
    };

    f.debug_struct(struct_name)
        .field("value", value)
        .field("addr", &signal.addr())
        .finish()
}

pub struct SignalRef<'a, T: ?Sized> {
    r: TraceRef<'a, T>,
    _drop: SignalRefDrop<'a, T>,
}

impl<T: ?Sized> Deref for SignalRef<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.r
    }
}

struct SignalRefDrop<'a, T: ?Sized>(&'a Signal<T>);

impl<T: ?Sized> Drop for SignalRefDrop<'_, T> {
    fn drop(&mut self) {
        let inner = &self.0.inner;
        // occupy the lock to prevent cyclic update
        let mut list = inner.delay_update_indexes.borrow_mut();
        for index in list.drain(..) {
            inner.dep_list.0[index].manual_update();
        }
    }
}
