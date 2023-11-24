use std::any::Any;

use super::{style_box::RawStyleGroup, StyleGroup};

#[derive(Clone)]
pub enum Branch<T, U> {
    ArmA(T),
    ArmB(U),
}

impl<T, U> RawStyleGroup for Branch<T, U>
where
    T: StyleGroup,
    U: StyleGroup,
{
    fn get_style_raw(&self, empty_option: &mut dyn Any) -> bool {
        match self {
            Self::ArmA(v) => v.get_style_raw(empty_option),
            Self::ArmB(v) => v.get_style_raw(empty_option),
        }
    }
}
