use std::any::Any;

use crate::{
    prim_element::{Size, SpaceConstraint},
    primitive::Point,
};

use super::Child as ChildStorage;

pub trait BlockLayout: Any {
    fn compute_layout(
        &self,
        children: LayoutChildren,
        constraint: Size<SpaceConstraint>,
    ) -> Size<f32>;
}

pub struct Child<'a>(&'a mut ChildStorage);
impl Child<'_> {
    pub fn set_location(&mut self, location: Point) {
        self.0.location = location;
    }

    pub fn measure(&mut self, constraint: Size<SpaceConstraint>) -> Size<f32> {
        let size = self.0.element.borrow_mut().layout(constraint);
        self.0.cached_layout = Some((constraint, size));
        size
    }
}

pub struct LayoutChildren<'a> {
    children: &'a mut [ChildStorage],
}

impl<'a> LayoutChildren<'a> {
    pub(super) fn new(children: &'a mut [ChildStorage]) -> Self {
        for child in children.iter_mut() {
            child.cached_layout = None;
            child.location = Point::ZERO;
        }

        Self { children }
    }

    pub fn iter(&mut self) -> impl Iterator<Item = Child<'_>> + use<'_, 'a> {
        self.children.iter_mut().map(Child)
    }

    pub fn get(&mut self, index: usize) -> Option<Child> {
        self.children.get_mut(index).map(Child)
    }

    pub fn len(&self) -> usize {
        self.children.len()
    }

    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }
}
