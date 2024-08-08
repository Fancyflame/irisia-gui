pub use self::{length::Length, pixel::Pixel, point::Point};

pub mod length;
pub mod pixel;
pub mod point;

pub type Result<T> = anyhow::Result<T>;
pub type Region = (Point, Point);
