pub use self::{length::Length, point::Point};

pub mod length;
pub mod point;

pub type Result<T> = anyhow::Result<T>;
pub type Region = (Point, Point);
