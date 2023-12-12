use std::any::Any;

use crate::{Style, StyleReader};

use super::{AnimaStyleGroup, StyleGroup};

pub trait RawStyleGroup {
    fn get_style_raw(&self, empty_option: &mut dyn Any) -> bool;
}

impl RawStyleGroup for () {
    fn get_style_raw(&self, _empty_option: &mut dyn Any) -> bool {
        false
    }
}

impl<T> RawStyleGroup for &T
where
    T: RawStyleGroup + ?Sized,
{
    fn get_style_raw(&self, empty_option: &mut dyn Any) -> bool {
        (**self).get_style_raw(empty_option)
    }
}

impl<T> StyleGroup for T
where
    T: RawStyleGroup + ?Sized,
{
    fn get_style<U: Style>(&self) -> Option<U> {
        let mut option: Option<U> = None;
        self.get_style_raw(&mut option);
        option
    }
}

pub trait IntoAnimaStyleGroup<_T> {
    type Output: AnimaStyleGroup;
    fn into_asg(self) -> Self::Output;
}

pub struct FromStyleGroup<T>(T);
impl<T> IntoAnimaStyleGroup<FromStyleGroup<T>> for T
where
    T: StyleGroup,
{
    type Output = FromStyleGroup<T>;

    fn into_asg(self) -> Self::Output {
        FromStyleGroup(self)
    }
}

impl<T> AnimaStyleGroup for FromStyleGroup<T>
where
    T: StyleGroup,
{
    fn read<Sr>(&self, _position: f32) -> Sr
    where
        Sr: StyleReader,
    {
        self.0.read()
    }
}

pub struct FromPosSgFn<T>(T);
impl<F, R> IntoAnimaStyleGroup<FromPosSgFn<F>> for F
where
    F: Fn(f32) -> R,
    R: StyleGroup,
{
    type Output = FromPosSgFn<F>;

    fn into_asg(self) -> Self::Output {
        FromPosSgFn(self)
    }
}

impl<F, R> AnimaStyleGroup for FromPosSgFn<F>
where
    F: Fn(f32) -> R,
    R: StyleGroup,
{
    fn read<Sr>(&self, position: f32) -> Sr
    where
        Sr: StyleReader,
    {
        (self.0)(position).read()
    }
}
