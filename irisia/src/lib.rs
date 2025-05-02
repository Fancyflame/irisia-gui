pub mod anyhow {
    pub use anyhow::*;
}

pub type Result<T> = anyhow::Result<T>;
type Handle<T> = Rc<RefCell<T>>;
type WeakHandle<T> = Weak<RefCell<T>>;

macro_rules! inner_error {
    ($($tt:tt)+) => {
        ::std::panic!("[IRISIA_INNER_ERROR {}: {}] {}", ::std::file!(), ::std::line!(), ::std::format!($($tt)+))
    };
}

pub mod application;
pub mod event;
pub mod hook;
pub mod log;
pub mod model;
pub mod prim_element;
pub mod primitive;

use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

pub use application::Window;
pub use event::Event;
pub use irisia_backend::{runtime::exit_app, skia_safe, start_runtime, winit, WinitWindow};
pub use irisia_macros::{build, build2, main, props, style, user_props, Event, PartialEq};
//pub use style::{ReadStyle, Style, WriteStyle};
