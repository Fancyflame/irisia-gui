use std::any::Any;

use super::{Style, StyleContainer};

#[derive(Clone)]
pub struct AddStyle<S>(pub(super) S);

impl<S> StyleContainer for AddStyle<S>
where
    S: Style,
{
    fn get_style<T: Style>(&self) -> Option<T> {
        (self as &dyn Any)
            .downcast_ref::<AddStyle<T>>()
            .map(|slf| slf.0.clone())
    }
}

impl<S: Style> AddStyle<S> {
    pub fn new(style: S) -> Self {
        AddStyle(style)
    }
}
