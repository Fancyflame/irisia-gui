pub trait UseStyle<T> {
    fn style_mut(&mut self) -> &mut T;
}

impl<T> UseStyle<T> for T {
    fn style_mut(&mut self) -> &mut T {
        self
    }
}
