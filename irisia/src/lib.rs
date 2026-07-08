pub type Result<T> = anyhow::Result<T>;
type Handle<T> = Rc<RefCell<T>>;
type WeakHandle<T> = Weak<RefCell<T>>;

macro_rules! inner_error {
    ($($tt:tt)+) => {
        ::std::panic!("[IRISIA_INNER_ERROR {}: {}] {}", ::std::file!(), ::std::line!(), ::std::format!($($tt)+))
    };
}

// mod application;
pub mod hook;
pub mod log;
// pub mod model;
// pub mod prim_element;
pub mod global;
pub mod primitive;

use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

pub use anyhow;
pub use hook::Signal;
pub use irisia_backend::{WinitWindow, runtime::exit_app, skia_safe, start_runtime, winit};
pub use irisia_macros::{Event, Property, build, main, style};
pub use primitive::{Corner, Point, Rect, Size};

pub trait Component: 'static {
    /// 当前组件正在初始化自身，现在暂时不能访问该entity上其他的组件
    fn on_initialize(&mut self, id: (), entity_id: ()) {
        let _ = (id, entity_id);
    }

    /// 所有组件已完成初始化，现在可以正常访问其他组件了
    fn on_mounted(&mut self) {}
}
