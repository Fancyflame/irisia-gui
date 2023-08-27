use std::any::Any;

use crate::Style;

use super::StyleContainer;

pub trait InsideStyleBox {
    fn get_style_raw(&self, empty_option: &mut dyn Any) -> bool;
}

impl InsideStyleBox for () {
    fn get_style_raw(&self, _empty_option: &mut dyn Any) -> bool {
        false
    }
}

impl<S: Style> InsideStyleBox for S {
    fn get_style_raw(&self, empty_option: &mut dyn Any) -> bool {
        if let Some(this) = empty_option.downcast_mut::<Option<Self>>() {
            *this = Some(self.clone());
            true
        } else {
            false
        }
    }
}

impl<T> StyleContainer for T
where
    T: InsideStyleBox + ?Sized,
{
    fn get_style<U: Style>(&self) -> Option<U> {
        let mut option: Option<U> = None;
        self.get_style_raw(&mut option);
        option
    }
}

pub struct StyleBox(Box<dyn InsideStyleBox>);

impl InsideStyleBox for StyleBox {
    fn get_style_raw(&self, empty_option: &mut dyn Any) -> bool {
        self.0.get_style_raw(empty_option)
    }
}
