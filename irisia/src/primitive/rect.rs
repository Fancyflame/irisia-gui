#[derive(Clone, Copy, Default)]
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
}
