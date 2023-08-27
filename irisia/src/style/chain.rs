use std::any::Any;

use super::{style_box::InsideStyleBox, StyleContainer};

#[derive(Clone)]
pub struct Chain<A, B> {
    former: A,
    latter: B,
}

impl<A, B> InsideStyleBox for Chain<A, B>
where
    A: StyleContainer,
    B: StyleContainer,
{
    fn get_style_raw(&self, empty_option: &mut dyn Any) -> bool {
        self.former.get_style_raw(empty_option) || self.latter.get_style_raw(empty_option)
    }
}

impl<A, B> Chain<A, B>
where
    A: StyleContainer,
    B: StyleContainer,
{
    pub fn new(f: A, l: B) -> Self {
        Chain {
            former: f,
            latter: l,
        }
    }
}
