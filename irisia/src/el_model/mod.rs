use std::rc::Rc;

use anyhow::anyhow;

use crate::Result;

pub(crate) use self::element_model::RenderOn;
pub use self::{
    element_model::{EMCreateCtx, ElementAccess, ElementModel},
    layer::LayerRebuilder,
    watcher::ElInputWatcher,
};

mod element_model;
pub(crate) mod layer;
mod watcher;

pub type SharedEM<El> = Rc<ElementModel<El>>;

#[allow(unused)]
fn panic_on_debug(msg: &str) -> Result<()> {
    if cfg!(debug_assertions) {
        panic!("inner error: {}", msg);
    } else {
        Err(anyhow!("{}", msg))
    }
}
