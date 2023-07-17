pub mod impls;

use crate::{primitive::Region, style::StyleContainer};

/// Custom update methods.
pub trait StateUpdate<T> {
    /// Update the state, returns whether
    /// the new state is equivalent to the previous.
    ///
    /// - `updater`: The new state.
    /// - `equality_matters`: Whether the return value is matters.
    /// If not, you can return `false` directly without checking the equality
    /// which will cost nothing more than `true`.
    /// - `return`: Whether the state has changed. Return `false` is always
    /// correct, but may cause unnecessary redrawing.
    fn state_update(&mut self, updater: T, equality_matters: bool) -> bool;
}

pub trait ElProps: Default + 'static {
    fn update_style(&mut self, _style: impl StyleContainer) {}
    fn update_draw_region(&mut self, _reg: Region) {}
}
