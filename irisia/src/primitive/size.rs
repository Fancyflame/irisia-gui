use irisia_backend::winit::dpi::PhysicalSize;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Size<T> {
    pub width: T,
    pub height: T,
}

impl_two_dimensions!(Size width height);

impl<T> From<PhysicalSize<T>> for Size<T> {
    fn from(value: PhysicalSize<T>) -> Self {
        Size {
            width: value.width,
            height: value.height,
        }
    }
}
