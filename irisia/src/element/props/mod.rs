pub use self::{
    defaulter::{Defaulter, PropInitialized, PropNotInitialized},
    help_create::HelpCreate,
    help_update::HelpUpdate,
    set_std_styles::SetStdStyles,
};

pub struct CallUpdater;
pub struct MoveOwnership;
pub struct ReadStyle;

mod defaulter;
mod help_create;
mod help_update;
mod set_std_styles;

pub trait PropsUpdateResult {
    type UpdateResult;
}

pub trait PropsUpdateWith<T>: PropsUpdateResult {
    fn props_update_with(&mut self, updater: T) -> Self::UpdateResult;
    fn props_create_with(updater: T) -> Self;
}
