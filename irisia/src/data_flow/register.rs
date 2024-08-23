use std::{
    cell::RefMut,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use super::{
    trace_cell::{TraceCell, TraceRef},
    Listener, ListenerList, ReadRef, Readable,
};

pub type RcReg<T> = Rc<Register<T>>;

const BORROW_ERROR: &str = "cannot mutate data inside the register, \
    because the write guard is still held somewhere. please note that \
    you should drop the write guard immediately as long as you do \
    not use it";

pub struct Register<T> {
    data: TraceCell<T>,
    listeners: ListenerList,
}

pub struct WriteGuard<'a, T: ?Sized> {
    listeners: &'a ListenerList,
    r: Option<TraceRef<'a, RefMut<'a, T>>>,
}

impl<T> Register<T> {
    pub fn write(&self) -> WriteGuard<T> {
        WriteGuard {
            listeners: &self.listeners,
            r: Some(self.data.borrow_mut().expect(BORROW_ERROR)),
        }
    }

    pub fn set(&self, value: T) {
        *self.data.borrow_mut().expect(BORROW_ERROR) = value;
        self.listeners.wake_all();
    }
}

impl<T> Readable for Register<T> {
    type Data = T;

    fn read(&self) -> ReadRef<Self::Data> {
        self.listeners.capture_caller();
        ReadRef::CellRef(self.data.borrow().unwrap())
    }

    fn pipe(&self, listen_end: Listener) {
        self.listeners.watch(&listen_end)
    }
}

pub fn register<T>(data: T) -> Rc<Register<T>>
where
    T: 'static,
{
    Rc::new(Register {
        data: TraceCell::new(data),
        listeners: ListenerList::new(),
    })
}

impl<T: ?Sized> Drop for WriteGuard<'_, T> {
    fn drop(&mut self) {
        self.r.take();
        self.listeners.wake_all()
    }
}

impl<T: ?Sized> Deref for WriteGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.r.as_ref().unwrap()
    }
}

impl<T: ?Sized> DerefMut for WriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.r.as_mut().unwrap()
    }
}
