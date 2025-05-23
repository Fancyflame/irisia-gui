#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct Corner<T> {
    pub left_top: T,
    pub right_top: T,
    pub left_bottom: T,
    pub right_bottom: T,
}

impl_mul_dimensions!(Corner left_top right_top left_bottom right_bottom);

impl<T> Corner<T> {
    pub const fn top_bottom(top: T, bottom: T) -> Self
    where
        T: Copy,
    {
        Corner {
            left_top: top,
            right_top: top,
            left_bottom: bottom,
            right_bottom: bottom,
        }
    }

    pub const fn left_right(left: T, right: T) -> Self
    where
        T: Copy,
    {
        Corner {
            left_top: left,
            left_bottom: left,
            right_top: right,
            right_bottom: right,
        }
    }
}
