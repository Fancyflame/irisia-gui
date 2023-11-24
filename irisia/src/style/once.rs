use crate::Style;

use super::style_box::RawStyleGroup;

#[derive(Clone)]
pub struct Once<T>(pub T);

impl<T> RawStyleGroup for Once<T>
where
    T: Style,
{
    fn get_style_raw(&self, empty_option: &mut dyn std::any::Any) -> bool {
        if let Some(this) = empty_option.downcast_mut::<Option<T>>() {
            *this = Some(self.0.clone());
            true
        } else {
            false
        }
    }
}
