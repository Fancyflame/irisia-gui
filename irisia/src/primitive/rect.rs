use irisia_backend::skia_safe;

use super::{Point, Region, Size};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct Rect<T> {
    pub left: T,
    pub top: T,
    pub right: T,
    pub bottom: T,
}

impl_mul_dimensions!(Rect left top right bottom);

impl<T> Rect<T> {
    pub const fn xy(x: T, y: T) -> Self
    where
        T: Copy,
    {
        Rect {
            left: x,
            top: y,
            right: x,
            bottom: y,
        }
    }

    pub fn as_border_size(self) -> Point<(T, T)> {
        Point {
            x: (self.left, self.right),
            y: (self.top, self.bottom),
        }
    }
}

impl Rect<f32> {
    pub(crate) const fn to_skia_rect(self) -> skia_safe::Rect {
        skia_safe::Rect {
            left: self.left,
            top: self.top,
            right: self.right,
            bottom: self.bottom,
        }
    }

    pub(crate) fn round_to_skia_irect(self) -> skia_safe::IRect {
        skia_safe::IRect {
            left: self.left.floor() as _,
            top: self.top.floor() as _,
            right: self.right.ceil() as _,
            bottom: self.bottom.ceil() as _,
        }
    }

    pub(crate) const fn to_lagacy_region(self) -> Region {
        Region {
            left_top: Point {
                x: self.left,
                y: self.top,
            },
            right_bottom: Point {
                x: self.right,
                y: self.bottom,
            },
        }
    }

    pub const fn get_size(&self) -> Size<f32> {
        Size {
            width: self.right - self.left,
            height: self.bottom - self.top,
        }
    }

    pub const fn get_location(&self) -> Point<f32> {
        Point {
            x: self.left,
            y: self.top,
        }
    }

    pub const fn from_location_size(location: Point<f32>, size: Size<f32>) -> Self {
        Self {
            left: location.x,
            top: location.y,
            right: location.x + size.width,
            bottom: location.y + size.height,
        }
    }

    pub const fn inset(mut self, inset: Rect<f32>) -> Self {
        self.left += inset.left;
        self.top += inset.top;
        self.right -= inset.right;
        self.bottom -= inset.bottom;
        self
    }
}
