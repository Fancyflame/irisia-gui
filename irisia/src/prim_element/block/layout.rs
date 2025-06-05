use std::any::Any;

use crate::{
    prim_element::{
        RenderTreeExt,
        block::Child,
        layout::{LayoutInput, SpaceConstraint},
    },
    primitive::{
        Point,
        length::{LengthStandard, LengthStandardGlobalPart},
        size::Size,
    },
};

pub trait BlockLayout<Cd>: Any {
    fn compute_layout(
        &self,
        children: LayoutChildren<Cd>,
        constraint: Size<SpaceConstraint>,
    ) -> Size<f32>;
}

pub struct LayoutChild<'a, Cd> {
    child: &'a Child<Cd>,
    length_standard: Size<LengthStandardGlobalPart>,
}

impl<'a, Cd> LayoutChild<'a, Cd> {
    pub fn data(&self) -> &'a Cd {
        &self.child.child_data
    }

    pub fn compute_layout(
        &self,
        constraint: Size<SpaceConstraint>,
        percentage_reference: Size<f32>,
    ) -> Size<f32> {
        self.child
            .element
            .borrow_mut()
            .compute_layout_cached(LayoutInput {
                constraint,
                length_standard: self.length_standard.map_with(
                    percentage_reference,
                    |global, pr| LengthStandard {
                        global,
                        percentage_reference: pr,
                    },
                ),
            })
    }

    pub fn set_location(&self, location: Point<f32>) {
        self.child
            .element
            .borrow_mut()
            .common_mut()
            .layout_output
            .location = location;
    }
}

pub struct LayoutChildren<'a, Cd> {
    children: &'a [Child<Cd>],
    length_standard: &'a Size<LengthStandard>,
}

impl<'a, Cd> LayoutChildren<'a, Cd> {
    pub(super) fn new(children: &'a [Child<Cd>], ls: &'a Size<LengthStandard>) -> Self {
        Self {
            children,
            length_standard: ls,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = LayoutChild<'_, Cd>> + use<'_, 'a, Cd> {
        self.children.iter().map(|child| LayoutChild {
            child,
            length_standard: self.length_standard.as_ref().map(|ls| ls.global),
        })
    }

    pub fn get(&self, index: usize) -> LayoutChild<Cd> {
        LayoutChild {
            child: self
                .children
                .get(index)
                .expect("child id {index} is out of bounds"),
            length_standard: self.length_standard.as_ref().map(|ls| ls.global),
        }
    }

    pub fn len(&self) -> usize {
        self.children.len()
    }

    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    pub fn parent_size(&self) -> Size<f32> {
        self.length_standard.map(|ls| ls.percentage_reference)
    }

    pub fn length_standard(&self) -> &Size<LengthStandard> {
        &self.length_standard
    }
}

impl<Cd> Drop for LayoutChildren<'_, Cd> {
    fn drop(&mut self) {
        for child in self.children {
            child.element.borrow_mut().set_layout_completed();
        }
    }
}

#[derive(Clone, Copy)]
pub struct DefaultLayouter;

impl<Cd> BlockLayout<Cd> for DefaultLayouter {
    fn compute_layout(
        &self,
        children: LayoutChildren<Cd>,
        constraint: Size<SpaceConstraint>,
    ) -> Size<f32> {
        let mut final_size = Size {
            width: 0.0,
            height: 0.0,
        };

        let pct_ref = constraint.as_ref().map(|cons| {
            if let SpaceConstraint::Exact(exact) = cons {
                *exact
            } else {
                0.0
            }
        });

        for child in children.iter() {
            let this_size = child.compute_layout(constraint, pct_ref);
            final_size.width = this_size.width.max(final_size.width);
            final_size.height = this_size.height.max(final_size.height);
        }

        final_size.map_with(constraint, |len, cons| {
            if let SpaceConstraint::Exact(exact) = cons {
                exact
            } else {
                len
            }
        })
    }
}
