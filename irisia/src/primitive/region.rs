use irisia_backend::skia_safe::IRect;

use super::Point;

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Region {
    pub left_top: Point,
    pub right_bottom: Point,
}

impl Region {
    pub const fn new(left_top: Point, right_bottom: Point) -> Self {
        Region {
            left_top,
            right_bottom,
        }
    }

    pub const fn contains(&self, rhs: Self) -> bool {
        self.left_top.abs_le(rhs.left_top) && self.right_bottom.abs_ge(rhs.right_bottom)
    }

    pub const fn contains_point(&self, point: Point) -> bool {
        point.abs_ge(self.left_top) && point.abs_le(self.right_bottom)
    }

    pub const fn intersects(&self, rhs: Self) -> bool {
        !(self.left_top.x >= rhs.right_bottom.x
            || self.left_top.y >= rhs.right_bottom.y
            || self.right_bottom.x <= rhs.left_top.x
            || self.right_bottom.y <= rhs.left_top.y)
    }

    pub const fn is_empty(&self) -> bool {
        self.left_top.x == self.right_bottom.x && self.left_top.y == self.right_bottom.y
    }

    pub fn ceil_to_irect(&self) -> IRect {
        IRect::from_ltrb(
            self.left_top.x.floor() as _,
            self.left_top.y.floor() as _,
            self.right_bottom.x.ceil() as _,
            self.right_bottom.y.ceil() as _,
        )
    }
}
