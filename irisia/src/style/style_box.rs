use std::any::Any;

use crate::{Style, StyleReader};

use super::StyleGroup;

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

trait StyleBoxInner: Fn(&mut dyn FnMut(&dyn RawStyleGroup), f32) + 'static {
    fn as_any(&mut self) -> &mut dyn Any;
}

impl<T> StyleBoxInner for T
where
    T: Fn(&mut dyn FnMut(&dyn RawStyleGroup), f32) + Any + 'static,
{
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct StyleBox(Box<dyn StyleBoxInner>);

impl StyleBox {
    pub fn new<T, _Sln>(into_sg: T) -> Self
    where
        T: IntoStyleBox<_Sln>,
    {
        into_sg.into_box()
    }

    fn update<T: StyleBoxInner>(&mut self, sbi: T) {
        match self.0.as_any().downcast_mut::<T>() {
            Some(place) => *place = sbi,
            None => self.0 = Box::new(sbi),
        }
    }

    pub fn read<Sr: StyleReader>(&self) -> Sr {
        self.read_at(0.)
    }

    pub fn read_at<Sr: StyleReader>(&self, prog: f32) -> Sr {
        let mut option: Option<Sr> = None;
        let mut assign = |sg: &dyn RawStyleGroup| {
            option = Some(sg.read());
        };
        (self.0)(&mut assign, prog);
        option.unwrap()
    }
}

pub trait IntoStyleBox<_T> {
    fn into_box(self) -> StyleBox;
    fn update(self, b: &mut StyleBox);
}

pub struct FromStyleGroup;
impl<T> IntoStyleBox<FromStyleGroup> for T
where
    T: StyleGroup + 'static,
{
    fn into_box(self) -> StyleBox {
        StyleBox(Box::new(style_box_from_sg(self)))
    }

    fn update(self, b: &mut StyleBox) {
        b.update(style_box_from_sg(self))
    }
}

fn style_box_from_sg<T: StyleGroup + 'static>(sg: T) -> impl StyleBoxInner {
    move |read_style, _| read_style(&sg)
}

pub struct FromAnimaStyleGroup;
impl<F, R> IntoStyleBox<FromAnimaStyleGroup> for F
where
    F: Fn(f32) -> R + 'static,
    R: StyleGroup,
{
    fn into_box(self) -> StyleBox {
        StyleBox(Box::new(style_box_from_asg(self)))
    }

    fn update(self, b: &mut StyleBox) {
        b.update(style_box_from_asg(self))
    }
}

fn style_box_from_asg<F, R>(sg: F) -> impl StyleBoxInner
where
    F: Fn(f32) -> R + 'static,
    R: StyleGroup,
{
    move |read_style, progress| read_style(&sg(progress))
}
