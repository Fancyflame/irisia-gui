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

pub mod branch;
pub mod chain;
pub mod once;
pub mod reader;
pub(crate) mod style_box;

use self::style_box::InsideStyleBox;
pub use self::{branch::Branch, chain::Chain, once::Once, style_box::StyleBox};

use crate::{self as irisia, primitive::Pixel, Style as DeriveStyle};
use irisia_backend::skia_safe::Color;

pub use reader::StyleReader;

pub trait Style: Clone + 'static {}

#[derive(Debug, DeriveStyle, Clone, Copy, PartialEq)]
#[style(from)]
pub struct StyleColor(pub Color);

#[derive(Debug, DeriveStyle, Clone, Copy, PartialEq)]
pub enum XAxisBound {
    #[style(option)]
    Left(#[style(default)] Pixel),

    #[style(option)]
    Right(#[style(default)] Pixel),
}

pub trait StyleContainer: InsideStyleBox {
    fn get_style<T: Style>(&self) -> Option<T>;

    fn read<S>(&self) -> S
    where
        S: StyleReader,
    {
        S::read_style(self)
    }
}
