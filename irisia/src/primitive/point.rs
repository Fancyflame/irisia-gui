use irisia_backend::{skia_safe::Point as SkiaPoint, winit::dpi::PhysicalPosition};

use crate::primitive::line::Line;

use super::{Size, rect::Rect};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Point<T = f32> {
    pub x: T,
    pub y: T,
}

impl_mul_dimensions!(Point x y);

impl<T> Point<T> {
    pub fn split_hv_to_rect(&self) -> Rect<T>
    where
        T: Clone,
    {
        Rect {
            left: self.x.clone(),
            top: self.y.clone(),
            right: self.x.clone(),
            bottom: self.y.clone(),
        }
    }

    pub fn to_size(self) -> Size<T> {
        Size {
            width: self.x,
            height: self.y,
        }
    }
}

impl<T> Point<Line<T>> {
    pub fn merge_hv_components(self) -> Rect<T> {
        let Self {
            x: Line {
                start: left,
                end: right,
            },
            y: Line {
                start: top,
                end: bottom,
            },
        } = self;

        Rect {
            left,
            top,
            right,
            bottom,
        }
    }
}

impl Point {
    pub const ZERO: Self = Point { x: 0.0, y: 0.0 };

    pub fn abs_diff(self, other: Self) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    /// Absolutely greater than or equals
    pub const fn abs_ge(self, other: Self) -> bool {
        self.x >= other.x && self.y >= other.y
    }

    /// Absolutely less than or equals
    pub const fn abs_le(self, other: Self) -> bool {
        self.x <= other.x && self.y <= other.y
    }
}

impl From<PhysicalPosition<f64>> for Point {
    fn from(value: PhysicalPosition<f64>) -> Self {
        assert!(value.x < f32::MAX as f64);
        assert!(value.y < f32::MAX as f64);
        Point {
            x: value.x as f32,
            y: value.y as f32,
        }
    }
}

impl From<Point> for SkiaPoint {
    fn from(value: Point) -> Self {
        SkiaPoint {
            x: value.x,
            y: value.y,
        }
    }
}
