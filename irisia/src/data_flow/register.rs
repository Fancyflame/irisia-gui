use std::{
    backtrace::Backtrace,
    cell::{Ref, RefMut},
    ops::{Deref, DerefMut},
    rc::Rc,
};

use super::{
    trace_cell::{TraceCell, TraceRef},
    Listener, ListenerList, Readable,
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

pub struct WriteGuard<'a, T> {
    listeners: &'a ListenerList,
    r: Option<TraceRef<'a, RefMut<'a, T>>>,
}

impl<T> Register<T> {
    pub fn write(&self) -> WriteGuard<T> {
        WriteGuard {
            listeners: &self.listeners,
            r: Some(
                self.data
                    .borrow_mut(Backtrace::force_capture())
                    .expect(BORROW_ERROR),
            ),
        }
    }

    pub fn set(&self, value: T) {
        let bt = Backtrace::force_capture();
        *self.data.borrow_mut(bt).expect(BORROW_ERROR) = value;
        self.listeners.wake_all();
    }
}

impl<T> Readable for Register<T> {
    type Data = T;

    fn read(&self) -> TraceRef<Ref<Self::Data>> {
        let bt = Backtrace::force_capture();
        self.listeners.capture_caller();
        self.data.borrow(bt).unwrap()
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

impl<T> Drop for WriteGuard<'_, T> {
    fn drop(&mut self) {
        self.r.take();
        self.listeners.wake_all()
    }
}

impl<T> Deref for WriteGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.r.as_ref().unwrap()
    }
}

impl<T> DerefMut for WriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.r.as_mut().unwrap()
    }
}
