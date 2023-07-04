pub mod diff_watcher;
pub mod impls;

pub use impls::StyleWatcher;

use crate::{primitive::Region, style::StyleContainer};

pub trait StateStore: 'static {
    type Store: Default + 'static;
}

pub trait StateUpdate<T>: StateStore {
    fn state_update(state: &mut Self::Store, updater: T);
}

pub trait ElProps: Default + 'static {
    fn update_style(&mut self, style: impl StyleContainer);
    fn update_draw_region(&mut self, reg: Region);
}
