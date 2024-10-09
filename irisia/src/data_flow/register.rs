use std::{
    ops::{Deref, DerefMut},
    rc::{Rc, Weak},
};

use super::{
    deps::Listener,
    trace_cell::{TraceCell, TraceMut},
    Listenable, ListenerList, ReadRef, ReadWire, Readable,
};

pub type RcReg<T> = Rc<Register<T>>;

const BORROW_ERROR: &str = "cannot mutate data inside the register, \
    because the write guard is still held somewhere. please note that \
    you should drop the write guard immediately as long as you do \
    not use it";

pub struct Register<T> {
    inner: Rc<Inner<T>>,
}

struct Inner<T> {
    data: TraceCell<T>,
    listeners: ListenerList,
    this: Weak<dyn Listenable>,
}

impl<T> Register<T> {
    pub fn register(data: T) -> Self
    where
        T: 'static,
    {
        let inner = Rc::new_cyclic(|this| Inner {
            data: TraceCell::new(data),
            listeners: ListenerList::new(),
            this: this.clone(),
        });

        Self { inner }
    }

    pub fn write(&self) -> WriteGuard<T> {
        WriteGuard {
            listeners: &self.inner.listeners,
            r: self.inner.data.borrow_mut().expect(BORROW_ERROR),
        }
    }

    pub fn set(&self, value: T) {
        *self.write() = value;
    }
}

impl<T> Readable for Inner<T> {
    type Data = T;

    fn read(&self) -> ReadRef<Self::Data> {
        self.listeners.capture_caller(&self.this.upgrade().unwrap());
        ReadRef::CellRef(self.data.borrow().unwrap())
    }
}

impl<T> Listenable for Inner<T> {
    fn add_listener(&self, listener: &Listener) {
        self.listeners.add_listener(listener);
    }

    fn remove_listener(&self, listener: &Listener) {
        self.listeners.remove_listener(listener);
    }
}

pub struct WriteGuard<'a, T: ?Sized> {
    // do not swap the field order
    r: TraceMut<'a, T>,
    listeners: WakeListeners<'a>,
}

impl<T: ?Sized> Deref for WriteGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.r
    }
}

impl<T: ?Sized> DerefMut for WriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.r
    }
}

struct WakeListeners<'a>(&'a ListenerList);

impl Drop for WakeListeners<'_> {
    fn drop(&mut self) {
        self.0.wake_all();
    }
}

impl<T> Deref for Register<T> {
    type Target = ReadWire<T>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
