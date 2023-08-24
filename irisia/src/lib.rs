pub mod anyhow {
    pub use anyhow::*;
}

pub type Result<T> = anyhow::Result<T>;

#[path = "macro_helper/mod.rs"]
mod __macro_helper;

macro_rules! inner_error {
    ($($tt:tt)+) => {
        ::std::panic!("[IRISIA_INNER_ERROR {}: {}] {}", ::std::file!(), ::std::line!(), ::std::format!($($tt)+))
    };
}

pub mod application;
pub(crate) mod dom;
pub mod element;
pub mod event;
pub mod log;
pub mod primitive;
pub mod structure;
pub mod style;
pub mod update_with;

#[doc(hidden)]
pub use __macro_helper::*;

pub use application::Window;
pub use element::Element;
pub use event::Event;
pub use irisia_backend::{
    runtime::exit_app, skia_safe, start_runtime, winit, StaticWindowEvent, WinitWindow,
};
pub use irisia_macros::{build, main, style, Event, Style, StyleReader};
pub use style::{reader::StyleReader, Style};
pub use update_with::UpdateWith;

//mod prop_test;
