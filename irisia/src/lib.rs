pub mod anyhow {
    pub use anyhow::*;
}

pub type Result<T> = anyhow::Result<T>;

#[doc(hidden)]
#[path = "macro_helper.rs"]
pub mod __macro_helper;

macro_rules! inner_error {
    ($($tt:tt)+) => {
        ::std::panic!("[IRISIA_INNER_ERROR {}: {}] {}", ::std::file!(), ::std::line!(), ::std::format!($($tt)+))
    };
}

pub mod application;
pub mod data_flow;
pub mod el_model;
pub mod element;
pub mod event;
pub mod log;
pub mod primitive;
pub mod structure;
pub mod style;

pub use application::Window;
pub use element::ElementInterfaces;
pub use event::Event;
pub use irisia_backend::{runtime::exit_app, skia_safe, start_runtime, winit, WinitWindow};
pub use irisia_macros::{
    build, define_style, main, style, user_props, Event, ReadStyle, WriteStyle,
};
pub use style::{ReadStyle, Style, WriteStyle};
