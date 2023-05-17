use std::ops::{Add, AddAssign, Sub, SubAssign};

pub type Region = (Point, Point);
pub type Result<T> = anyhow::Result<T>;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Point(pub f32, pub f32);

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
    pub fn abs_diff(self, other: Self) -> f32 {
        ((self.0 - other.0).powi(2) + (self.1 - other.1).powi(2)).sqrt()
    }

    /// Absolutely greater than or equals
    #[inline]
    pub fn abs_ge(self, other: Self) -> bool {
        self.0 >= other.0 && self.1 >= other.1
    }

    /// Absolutely less than or equals
    #[inline]
    pub fn abs_le(self, other: Self) -> bool {
        self.0 <= other.0 && self.1 <= other.1
    }
}

impl From<(f32, f32)> for Point {
    #[inline]
    fn from(f: (f32, f32)) -> Self {
        Point(f.0, f.1)
    }
}

impl From<Point> for (f32, f32) {
    fn from(v: Point) -> Self {
        (v.0, v.1)
    }
}

impl From<Point> for irisia_backend::skia_safe::Point {
    fn from(value: Point) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}
