pub mod computed;
pub mod data;
pub(crate) mod dep;
mod thread_guard;
pub mod watcher;

use std::rc::Rc;

pub use computed::Computed;
pub use data::Data;
pub use watcher::{Watchable, Watcher};
pub type RcWatchable<D> = Rc<dyn Watchable<D>>;
