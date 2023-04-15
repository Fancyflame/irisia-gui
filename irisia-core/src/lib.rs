pub mod anyhow {
    pub use anyhow::*;
}

pub type Result<T> = anyhow::Result<T>;

#[path = "macro_helper/mod.rs"]
mod __macro_helper;

pub mod application;
pub mod element;
pub mod event;
pub mod primary;
pub mod structure;
pub mod style;

#[doc(hidden)]
pub use __macro_helper::*;

pub use application::Window;
pub use element::{Element, Frame};
pub use event::Event;
pub use irisia_backend::{
    runtime::exit_app, skia_safe, start_runtime, winit, StaticWindowEvent, WinitWindow,
};
pub use irisia_macros::*;
pub use structure::cache_box::CacheBox;
pub use style::{reader::StyleReader, Style};
