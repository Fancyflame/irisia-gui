pub use self::{length::Length, point::Point, region::Region};

#[macro_use]
mod vector2;

pub mod length;
pub mod point;
pub mod region;
pub mod size;

pub type Result<T> = anyhow::Result<T>;
