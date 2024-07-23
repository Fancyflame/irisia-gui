use std::{
    cell::Ref,
    rc::{Rc, Weak},
};

pub use self::wire::{wire, wire2, wire3};
use listener_list::ListenerList;
use map::Map;
use trace_cell::TraceRef;
use watcher::{watcher, Handle};

pub mod convert_from;
mod listener_list;
pub mod map;
pub mod observer;
pub mod register;
pub mod trace_cell;
pub mod watcher;
mod wire;

pub type ReadWire<T> = Rc<dyn Readable<Data = T>>;
pub type ReadRef<'a, T> = TraceRef<'a, Ref<'a, T>>;

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
    fn update(self: Rc<Self>) -> bool;
}
