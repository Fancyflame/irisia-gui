pub use self::{
    corner::Corner, length::Length, point::Point, rect::Rect, region::Region, size::Size,
};

#[macro_use]
mod mul_dimensions;

pub mod corner;
pub mod length;
pub mod line;
pub mod point;
pub mod rect;
pub mod region;
pub mod size;

pub type Result<T> = anyhow::Result<T>;
