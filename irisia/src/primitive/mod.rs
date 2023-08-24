pub use self::{pixel::Pixel, point::Point};

pub mod pixel;
pub mod point;

pub type Result<T> = anyhow::Result<T>;
pub type Region = (Point, Point);
