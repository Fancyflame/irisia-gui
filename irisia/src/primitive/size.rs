use irisia_backend::winit::dpi::PhysicalSize;

use super::Point;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Size<T> {
    pub width: T,
    pub height: T,
}

impl_mul_dimensions!(Size width height);

impl<T> Size<T> {
    pub fn to_point(self) -> Point<T> {
        Point {
            x: self.width,
            y: self.height,
        }
    }
}

impl<T> From<PhysicalSize<T>> for Size<T> {
    fn from(value: PhysicalSize<T>) -> Self {
        Size {
            width: value.width,
            height: value.height,
        }
    }
}

impl Size<f32> {
    pub const fn is_empty(&self) -> bool {
        self.width == 0.0 || self.height == 0.0
    }
}
