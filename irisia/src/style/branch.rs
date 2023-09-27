use std::any::Any;

use super::{style_box::InsideStyleBox, StyleContainer};

#[derive(Clone)]
pub enum Branch<T, U> {
    ArmA(T),
    ArmB(U),
}

impl<T, U> InsideStyleBox for Branch<T, U>
where
    T: StyleContainer,
    U: StyleContainer,
{
    fn get_style_raw(&self, empty_option: &mut dyn Any) -> bool {
        match self {
            Self::ArmA(v) => v.get_style_raw(empty_option),
            Self::ArmB(v) => v.get_style_raw(empty_option),
        }
    }
}
