use crate::Style;

use super::style_box::InsideStyleBox;

#[derive(Clone)]
pub struct Once<T>(pub T);

impl<T> InsideStyleBox for Once<T>
where
    T: Style,
{
    fn get_style_raw(&self, empty_option: &mut dyn std::any::Any) -> bool {
        if let Some(this) = empty_option.downcast_mut::<Option<Self>>() {
            *this = Some(self.clone());
            true
        } else {
            false
        }
    }
}
