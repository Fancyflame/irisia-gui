pub mod diff_watcher;
pub mod impls;

pub use impls::StyleWatcher;

pub trait StateStore: 'static {
    type Store: 'static;
}

pub trait StateUpdate<T>: StateStore {
    fn state_created(updater: T) -> Self::Store;
    fn state_changed(state: &mut Self::Store, updater: T);
}
