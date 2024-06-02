use std::{
    cell::Ref,
    rc::{Rc, Weak},
};

pub use self::wire::{wire, wire2, wire3};
use listener_list::ListenerList;
use watcher::{watcher, Handle};

pub mod convert_from;
mod listener_list;
pub mod observer;
pub mod register;
pub mod watcher;
mod wire;

pub type ReadWire<T> = Rc<dyn Readable<Data = T>>;

#[derive(Clone)]
pub enum Listener {
    Weak(Weak<dyn Wakeable>),
    Rc(Rc<dyn Wakeable>),
}

pub trait Readable {
    type Data: ?Sized;
    fn read(&self) -> Ref<Self::Data>;
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
}

impl<T> ReadableExt for T where T: Readable + ?Sized + 'static {}

pub trait Wakeable {
    fn update(self: Rc<Self>) -> bool;
}
