pub mod anyhow {
    pub use anyhow::*;
}

pub type Result<T> = anyhow::Result<T>;

#[doc(hidden)]
#[path = "private/mod.rs"]
pub mod __private;

macro_rules! inner_error {
    ($($tt:tt)+) => {
        ::std::panic!("[IRISIA_INNER_ERROR {}: {}] {}", ::std::file!(), ::std::line!(), ::std::format!($($tt)+))
    };
}

pub mod application;
pub mod dep_watch;
pub mod dom;
pub mod element;
pub mod event;
pub mod log;
pub mod primitive;
pub mod structure;
pub mod style;
pub mod update_with;

pub use application::Window;
pub use dom::ChildNodes;
pub use element::Element;
pub use event::Event;
pub use irisia_backend::{runtime::exit_app, skia_safe, start_runtime, winit, WinitWindow};
pub use irisia_macros::{main, props, style, Event, Style, StyleReader};
pub use style::{reader::StyleReader, Style, StyleGroup};
pub use update_with::UpdateWith;
