#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Size<T> {
    pub width: T,
    pub height: T,
}

impl_two_dimensions!(Size width height);
