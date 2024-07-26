use std::any::Any;

use super::Style;

pub struct StyleBuffer<'a>(pub(super) &'a mut dyn Any);

impl StyleBuffer<'_> {
    #[inline(always)]
    pub fn write<T: Style>(&mut self, style: &T) {
        if let Some(opt) = self.0.downcast_mut::<Option<T>>() {
            *opt = Some(style.clone());
        }
    }
}

pub trait ReadStyle {
    fn read_style_into(&self, buf: &mut StyleBuffer);
}

impl<F> ReadStyle for F
where
    F: Fn(&mut StyleBuffer),
{
    fn read_style_into(&self, buf: &mut StyleBuffer) {
        self(buf)
    }
}

impl ReadStyle for () {
    fn read_style_into(&self, _: &mut StyleBuffer) {}
}
