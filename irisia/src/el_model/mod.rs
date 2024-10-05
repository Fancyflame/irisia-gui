use anyhow::anyhow;

use crate::Result;

pub use self::element_model::{EMCreateCtx, ElementAccess, ElementModel};

mod element_model;

#[allow(unused)]
fn panic_on_debug(msg: &str) -> Result<()> {
    if cfg!(debug_assertions) {
        panic!("inner error: {}", msg);
    } else {
        Err(anyhow!("{}", msg))
    }
}
