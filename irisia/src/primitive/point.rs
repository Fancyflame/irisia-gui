use std::ops::{Add, AddAssign, Sub, SubAssign};

use irisia_backend::{skia_safe::Point as SkiaPoint, winit::dpi::PhysicalPosition};

use super::Pixel;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Point(pub Pixel, pub Pixel);

impl Add for Point {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Point(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl AddAssign for Point {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Point {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Point(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl SubAssign for Point {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Point {
    pub fn abs_diff(self, other: Self) -> Pixel {
        Pixel(((self.0 - other.0).0.powi(2) + (self.1 - other.1).0.powi(2)).sqrt())
    }

    /// Absolutely greater than or equals
    pub fn abs_ge(self, other: Self) -> bool {
        self.0 >= other.0 && self.1 >= other.1
    }

    /// Absolutely less than or equals
    pub fn abs_le(self, other: Self) -> bool {
        self.0 <= other.0 && self.1 <= other.1
    }
}

impl From<(Pixel, Pixel)> for Point {
    #[inline]
    fn from(f: (Pixel, Pixel)) -> Self {
        Point(f.0, f.1)
    }
}

impl From<Point> for (Pixel, Pixel) {
    fn from(v: Point) -> Self {
        (v.0, v.1)
    }
}

impl From<PhysicalPosition<f64>> for Point {
    fn from(value: PhysicalPosition<f64>) -> Self {
        Point(
            Pixel::from_physical(value.x as _),
            Pixel::from_physical(value.y as _),
        )
    }
}

impl From<Point> for SkiaPoint {
    fn from(value: Point) -> Self {
        SkiaPoint {
            x: value.0.to_physical(),
            y: value.1.to_physical(),
        }
    }
}
