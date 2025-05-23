use std::any::Any;

use crate::{
    prim_element::{
        RenderTreeExt,
        layout::{FinalLayout, SpaceConstraint},
    },
    primitive::{length::LengthStandard, size::Size},
};

use super::Child as ChildStorage;

pub trait BlockLayout: Any {
    fn compute_layout(
        &self,
        children: LayoutChildren,
        constraint: Size<SpaceConstraint>,
    ) -> Size<f32>;
}

pub struct Child<'a> {
    child: &'a ChildStorage,
    length_standard: &'a Size<LengthStandard>,
}

impl Child<'_> {
    pub fn measure(&self, constraint: Size<SpaceConstraint>) -> Size<f32> {
        self.child
            .element
            .borrow_mut()
            .compute_layout_cached(constraint, *self.length_standard)
    }

    pub fn set_final_layout(&self, final_layout: Option<FinalLayout>) {
        self.child
            .element
            .borrow_mut()
            .set_final_layout(final_layout);
    }
}

pub struct LayoutChildren<'a> {
    children: &'a [ChildStorage],
    length_standard: &'a Size<LengthStandard>,
}

impl<'a> LayoutChildren<'a> {
    pub(super) fn new(children: &'a [ChildStorage], ls: &'a Size<LengthStandard>) -> Self {
        Self {
            children,
            length_standard: ls,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = Child<'_>> + use<'_, 'a> {
        self.children.iter().map(|child| Child {
            child,
            length_standard: self.length_standard,
        })
    }

    pub fn get(&self, index: usize) -> Option<Child> {
        self.children.get(index).map(|child| Child {
            child,
            length_standard: self.length_standard,
        })
    }

    pub fn len(&self) -> usize {
        self.children.len()
    }

    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }
}

#[derive(Clone, Copy)]
pub struct DefaultLayouter;

impl BlockLayout for DefaultLayouter {
    fn compute_layout(
        &self,
        children: LayoutChildren,
        constraint: Size<SpaceConstraint>,
    ) -> Size<f32> {
        let mut final_size = Size {
            width: 0.0,
            height: 0.0,
        };

        for child in children.iter() {
            let this_size = child.measure(constraint);
            final_size.width = this_size.width.max(final_size.width);
            final_size.height = this_size.height.max(final_size.height);
        }

        final_size
    }
}
