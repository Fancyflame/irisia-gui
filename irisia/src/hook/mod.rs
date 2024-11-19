use std::ops::Deref;

pub use listener::Listener;

pub mod consumer;
pub mod listener;
pub mod memo;
mod provider_wrapper;
pub mod read_many;
pub mod state;
pub mod trace_cell;
pub mod utils;

pub trait Provider {
    type Data: ?Sized;
    fn read(&self) -> Ref<Self::Data>;
    fn dependent(&self, listener: Listener);
}

pub enum Ref<'a, T: ?Sized> {
    Ref(&'a T),
    CellRef(trace_cell::TraceRef<'a, T>),
}

impl<T: ?Sized> Deref for Ref<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Ref(r) => r,
            Self::CellRef(r) => &r,
        }
    }
}
