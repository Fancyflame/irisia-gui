use super::Point;

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Region {
    pub left_top: Point,
    pub right_bottom: Point,
}

impl Region {
    pub fn contains(&self, rhs: Self) -> bool {
        self.left_top.abs_le(rhs.left_top) && self.right_bottom.abs_ge(rhs.right_bottom)
    }

    pub fn contains_point(&self, point: Point) -> bool {
        point.abs_ge(self.left_top) && point.abs_le(self.right_bottom)
    }

    pub fn intersects(&self, rhs: Self) -> bool {
        !(self.left_top.0 >= rhs.right_bottom.0
            || self.left_top.1 >= rhs.right_bottom.1
            || self.right_bottom.0 <= rhs.left_top.0
            || self.right_bottom.1 <= rhs.left_top.1)
    }
}
