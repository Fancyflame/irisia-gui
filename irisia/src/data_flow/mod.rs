use std::{
    cell::Ref,
    ops::Deref,
    rc::{Rc, Weak},
};

pub use self::{
    const_data::{const_wire, const_wire_unsized},
    register::register,
    wire::{wire, wire2, wire3},
};
use listener_list::ListenerList;
use map::Map;
use trace_cell::TraceRef;
use watcher::{watcher, Handle};

pub mod const_data;
pub mod convert_from;
mod listener_list;
pub mod map;
pub mod observer;
pub mod register;
pub mod trace_cell;
pub mod watch_on_deref;
pub mod watcher;
pub mod wire;

pub type ReadWire<T> = Rc<dyn Readable<Data = T>>;

#[derive(Clone)]
pub enum Listener {
    Weak(Weak<dyn Wakeable>),
    Rc(Rc<dyn Wakeable>),
}

pub trait Readable {
    type Data: ?Sized;

    fn read(&self) -> ReadRef<Self::Data>;
    fn pipe(&self, listen_end: Listener);
}

pub trait ReadableExt: Readable + 'static {
    fn watch<F>(self: &Rc<Self>, mut watch_fn: F, call_immediately: bool) -> Handle
    where
        F: FnMut(&Rc<Self>, &Handle) + 'static,
    {
        let this = self.clone();
        let (watcher, handle) = watcher(move |handle| watch_fn(&this, handle), call_immediately);
        self.pipe(watcher);
        handle
    }

    fn map<R>(self, f: fn(&Self::Data) -> &R) -> Map<Self, Self::Data, R>
    where
        Self: Sized,
    {
        Map::new(self, f)
    }
}

impl<T> Readable for Rc<T>
where
    T: Readable + ?Sized,
{
    type Data = T::Data;

    fn read(&self) -> ReadRef<Self::Data> {
        (**self).read()
    }

    fn pipe(&self, listen_end: Listener) {
        (**self).pipe(listen_end)
    }
}

impl<T> ReadableExt for T where T: Readable + ?Sized + 'static {}

pub trait Wakeable {
    /// Return true if the wakeable should continue to be waked.
    fn update(self: Rc<Self>) -> bool;
}

pub enum ReadRef<'a, T>
where
    T: ?Sized,
{
    Ref(&'a T),
    CellRef(TraceRef<'a, Ref<'a, T>>),
}

impl<T> Deref for ReadRef<'_, T>
where
    T: ?Sized,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            ReadRef::Ref(r) => r,
            ReadRef::CellRef(r) => r,
        }
    }
}

impl<'a, T: ?Sized> ReadRef<'a, T> {
    pub fn clone(this: &Self) -> ReadRef<'a, T> {
        match this {
            ReadRef::Ref(r) => ReadRef::Ref(r),
            ReadRef::CellRef(r) => ReadRef::CellRef(TraceRef::clone(r)),
        }
    }

    pub fn map<U, F>(self, f: F) -> ReadRef<'a, U>
    where
        F: FnOnce(&T) -> &U,
        U: ?Sized,
    {
        match self {
            ReadRef::Ref(r) => ReadRef::Ref(f(r)),
            ReadRef::CellRef(r) => ReadRef::CellRef(TraceRef::map(r, f)),
        }
    }
}
