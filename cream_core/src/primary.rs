use std::ops::{Add, Sub};

pub type Region = (Point, Point);
pub type Result<T> = anyhow::Result<T>;

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Point(pub u32, pub u32);

impl Add for Point {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Point(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl Sub for Point {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Point(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl Point {
    /// Absolutely greater than
    #[inline]
    pub fn abs_gt(self, other: Self) -> bool {
        self.0 >= other.0 && self.1 >= other.1
    }

    /// Absolutely less than
    #[inline]
    pub fn abs_lt(self, other: Self) -> bool {
        self.0 <= other.0 && self.1 <= other.1
    }
}

impl From<(u32, u32)> for Point {
    #[inline]
    fn from(f: (u32, u32)) -> Self {
        Point(f.0, f.1)
    }
}

impl From<Point> for (u32, u32) {
    fn from(v: Point) -> Self {
        (v.0, v.1)
    }
}
