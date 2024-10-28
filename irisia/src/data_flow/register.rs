use std::{
    ops::{Deref, DerefMut},
    rc::{Rc, Weak},
};

use super::{
    trace_cell::{TraceCell, TraceMut},
    Listenable, ListenerList, ReadRef, ReadWire, Readable, ToListener, ToReadWire,
};

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
    pub fn new(data: T) -> Self
    where
        T: 'static,
    {
        let inner = Rc::new_cyclic(|this: &Weak<Inner<_>>| Inner {
            data: TraceCell::new(data),
            listeners: ListenerList::new(),
            this: this.clone(),
        });

        Self { inner }
    }

    #[inline]
    pub fn read(&self) -> ReadRef<T> {
        self.inner.read()
    }

    pub fn write(&self) -> WriteGuard<T> {
        WriteGuard {
            _wl: WakeListeners(&self.inner.listeners),
            r: self.inner.data.borrow_mut().expect(BORROW_ERROR),
        }
    }

    pub fn set(&self, value: T) {
        *self.write() = value;
    }
}

impl<T> Readable for Register<T> {
    type Data = T;

    fn read(&self) -> ReadRef<Self::Data> {
        self.inner.read()
    }

    fn ptr_as_id(&self) -> *const () {
        self.inner.ptr_as_id()
    }
}

impl<T> Listenable for Register<T> {
    fn add_listener(&self, listener: &dyn ToListener) {
        self.inner.add_listener(listener);
    }

    fn remove_listener(&self, listener: &dyn ToListener) {
        self.inner.remove_listener(listener);
    }
}

impl<T> Readable for Inner<T> {
    type Data = T;

    fn read(&self) -> ReadRef<Self::Data> {
        self.listeners.capture_caller(&self.this.upgrade().unwrap());
        ReadRef::CellRef(self.data.borrow().unwrap())
    }

    fn ptr_as_id(&self) -> *const () {
        self.this.as_ptr().cast()
    }
}

impl<T> Listenable for Inner<T> {
    fn add_listener(&self, listener: &dyn ToListener) {
        self.listeners.add_listener(listener.to_listener());
    }

    fn remove_listener(&self, listener: &dyn ToListener) {
        self.listeners.remove_listener(listener.to_listener());
    }
}

pub struct WriteGuard<'a, T: ?Sized> {
    // do not swap the field order
    r: TraceMut<'a, T>,
    _wl: WakeListeners<'a>,
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
        self.0.set_dirty();
        self.0.wake_all();
    }
}

impl<T: 'static> ToReadWire for Register<T> {
    type Data = T;
    fn to_read_wire(&self) -> ReadWire<Self::Data> {
        self.inner.clone()
    }
}

impl<T> Default for Register<T>
where
    T: Default + 'static,
{
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> Clone for Register<T> {
    fn clone(&self) -> Self {
        Register {
            inner: self.inner.clone(),
        }
    }
}
