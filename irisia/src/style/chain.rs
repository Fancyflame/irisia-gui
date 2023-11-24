use std::any::Any;

use super::{style_box::RawStyleGroup, StyleGroup};

#[derive(Clone)]
pub struct Chain<A, B> {
    pub former: A,
    pub latter: B,
}

impl<A, B> RawStyleGroup for Chain<A, B>
where
    A: StyleGroup,
    B: StyleGroup,
{
    fn get_style_raw(&self, empty_option: &mut dyn Any) -> bool {
        self.former.get_style_raw(empty_option) || self.latter.get_style_raw(empty_option)
    }
}

impl<A, B> Chain<A, B>
where
    A: StyleGroup,
    B: StyleGroup,
{
    pub fn new(f: A, l: B) -> Self {
        Chain {
            former: f,
            latter: l,
        }
    }
}
