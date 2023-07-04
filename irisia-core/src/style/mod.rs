//! ## 样式格式
//! 对于下列代码
//! ```no_run
//! irisia::style!{
//!     box_shadow:
//!         10px, 10px,
//!         .outset,
//!         .blur 20px,
//!         .color Color::SKYBLUE,
//!         .pass_tuple ("bar", 7px)
//!     ;
//! }
//! ```
//! 1. **`box_shadow`**:
//!   将会转换成类型储存在样式集结构体中，并在初始化时执行`StyleBoxShadow::style_create((Pixel(10), Pixel(10)))`
//!
//! 2. **`10px`**:
//!   `px`, `pct`等后缀属于框架内置的固定数字表示法。`10px`将转换成`irisia::Pixel(10)`
//!
//! 3. **`.outset`**:
//!   以`.`开头且无参数，`style.outset()`
//!
//! 4. **`.blur 20px`**:
//!   以`.`开头有参数，`style.blur(Pixel(20))`，至多只允许一个参数
//!

pub mod add_style;
pub mod branch;
pub mod chain;
pub mod pixel;
pub mod reader;

use std::any::Any;

use crate as irisia;
pub use add_style::*;
pub use branch::*;
pub use chain::*;
use irisia_backend::skia_safe::Color;
use irisia_macros::Style;
pub use pixel::Pixel;

use self::reader::StyleReader;

pub trait Style: Clone + 'static {}

#[derive(Debug, Style, Clone, PartialEq)]
#[irisia(style(from))]
pub struct StyleColor(pub Color);

#[derive(Clone, Copy)]
pub struct NoStyle;

pub trait StyleContainer: Clone {
    fn get_style<T: Style>(&self) -> Option<T>;

    fn read<S: StyleReader>(&self) -> S {
        S::read_style(self)
    }

    fn chain<T: StyleContainer>(self, style: T) -> Chain<Self, T>
    where
        Self: Sized,
    {
        Chain::new(self, style)
    }
}

impl StyleContainer for NoStyle {
    fn get_style<T: Style>(&self) -> Option<T> {
        None
    }
}

impl<S: Style> StyleContainer for S {
    fn get_style<T: Style>(&self) -> Option<T> {
        (self as &dyn Any).downcast_ref::<T>().cloned()
    }
}
