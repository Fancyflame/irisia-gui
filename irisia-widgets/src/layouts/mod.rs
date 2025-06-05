use std::{ops::Range, u64};

use irisia::{
    prim_element::{block::layout::LayoutChildren, layout::SpaceConstraint},
    primitive::{length::LengthStandard, Size},
};
use taffy::{CoreStyle, LayoutPartialTree, NodeId, TraversePartialTree};

pub use flexbox::*;

pub mod base_style;
pub mod flexbox;

struct MyTree<'a, Cts, Cds> {
    container_style: &'a Cts,
    children: LayoutChildren<'a, Cds>,
}

const PARENT_NODE_ID: NodeId = NodeId::new(u64::MAX);

impl<Cts, Cds> TraversePartialTree for MyTree<'_, Cts, Cds> {
    type ChildIter<'a>
        = ChildIter
    where
        Self: 'a;

    fn child_count(&self, parent_node_id: NodeId) -> usize {
        assert_parent_node(parent_node_id);
        self.children.len()
    }

    fn child_ids(&self, parent_node_id: NodeId) -> Self::ChildIter<'_> {
        assert_parent_node(parent_node_id);
        ChildIter::from_len(self.children.len())
    }

    fn get_child_id(&self, parent_node_id: NodeId, child_index: usize) -> NodeId {
        assert_parent_node(parent_node_id);
        NodeId::from(child_index)
    }
}

impl<Cts, Cds> LayoutPartialTree for MyTree<'_, Cts, Cds>
where
    for<'a> ResolvedStyle<'a, &'a Cts>: CoreStyle,
{
    type CoreContainerStyle<'a>
        = ResolvedStyle<'a, &'a Cts>
    where
        Self: 'a;

    fn get_core_container_style(&self, node_id: NodeId) -> Self::CoreContainerStyle<'_> {
        assert_parent_node(node_id);
        ResolvedStyle {
            length_standard: self.children.length_standard().as_ref(),
            style: self.container_style,
        }
    }

    fn compute_child_layout(
        &mut self,
        node_id: NodeId,
        inputs: taffy::LayoutInput,
    ) -> taffy::LayoutOutput {
        let child_index = usize::from(node_id);
        let (constraint, pct_ref) = taffy_layout_input_to_local(&inputs);
        let measured_size = self
            .children
            .get(child_index)
            .compute_layout(constraint, pct_ref);

        taffy::LayoutOutput {
            size: taffy::Size {
                width: measured_size.width,
                height: measured_size.height,
            },
            ..taffy::LayoutOutput::DEFAULT
        }
    }

    fn set_unrounded_layout(&mut self, node_id: NodeId, layout: &taffy::Layout) {
        let child_index = usize::from(node_id);
        self.children
            .get(child_index)
            .set_location(point_to_local(layout.location));
    }
}

fn taffy_layout_input_to_local(inputs: &taffy::LayoutInput) -> (Size<SpaceConstraint>, Size<f32>) {
    let constraint = size_to_local(inputs.available_space.zip_map(
        inputs.known_dimensions,
        |available_space, known_dimension| match known_dimension {
            Some(exact) => SpaceConstraint::Exact(exact),
            None => match available_space {
                taffy::AvailableSpace::Definite(available) => SpaceConstraint::Available(available),
                taffy::AvailableSpace::MaxContent => SpaceConstraint::MaxContent,
                taffy::AvailableSpace::MinContent => SpaceConstraint::MinContent,
            },
        },
    ));

    let percentage_ref = Size {
        width: inputs.known_dimensions.width,
        height: inputs.known_dimensions.height,
    }
    .map(|x| x.unwrap_or(0.0));

    (constraint, percentage_ref)
}

fn local_layout_input_to_taffy(
    constraints: Size<SpaceConstraint>,
    parent_size: Size<Option<f32>>,
) -> taffy::LayoutInput {
    use taffy::AvailableSpace;

    let Size {
        width: (as_w, kd_w),
        height: (as_h, kd_h),
    } = constraints.map(|cons| match cons {
        SpaceConstraint::Exact(exact) => (AvailableSpace::Definite(exact), Some(exact)),
        SpaceConstraint::Available(available) => (AvailableSpace::Definite(available), None),
        SpaceConstraint::MaxContent => (AvailableSpace::MaxContent, None),
        SpaceConstraint::MinContent => (AvailableSpace::MinContent, None),
    });

    taffy::LayoutInput {
        run_mode: taffy::RunMode::PerformLayout,
        sizing_mode: taffy::SizingMode::ContentSize,
        axis: taffy::RequestedAxis::Both,
        known_dimensions: taffy::Size {
            width: kd_w,
            height: kd_h,
        },
        parent_size: size_to_taffy(parent_size),
        available_space: taffy::Size {
            width: as_w,
            height: as_h,
        },
        vertical_margins_are_collapsible: taffy::Line::FALSE,
    }
}

fn assert_parent_node(node_id: NodeId) {
    assert_eq!(node_id, PARENT_NODE_ID, "parent node id must be `u64::MAX`");
}

struct ChildIter {
    children_range: Range<usize>,
}

impl ChildIter {
    fn from_len(len: usize) -> Self {
        assert!((len as u64) < u64::MAX);
        Self {
            children_range: 0..len,
        }
    }
}

impl Iterator for ChildIter {
    type Item = NodeId;
    fn next(&mut self) -> Option<Self::Item> {
        self.children_range.next().map(NodeId::from)
    }
}

macro_rules! taffy_local_geometry_cast {
    ($($taffy_to_local:ident $local_to_taffy:ident: $Type:ident $($field:ident)*,)*) => {
        $(
            #[allow(dead_code)]
            fn $taffy_to_local<T>(input: taffy::$Type<T>) -> irisia::$Type<T> {
                irisia::primitive::$Type {
                    $($field: input.$field,)*
                }
            }

            #[allow(dead_code)]
            fn $local_to_taffy<T>(input: irisia::$Type<T>) -> taffy::$Type<T> {
                taffy::$Type {
                    $($field: input.$field,)*
                }
            }
        )*
    };
}

taffy_local_geometry_cast![
    size_to_local size_to_taffy: Size width height,
    point_to_local point_to_taffy: Point x y,
    rect_to_local rect_to_taffy: Rect left top right bottom,
];

struct ResolvedStyle<'a, T> {
    length_standard: Size<&'a LengthStandard>,
    style: T,
}

impl<T> ResolvedStyle<'_, T> {
    fn wrap<'a, U>(&'a self, other: U) -> ResolvedStyle<'a, U> {
        ResolvedStyle {
            length_standard: self.length_standard,
            style: other,
        }
    }
}

trait CoreStyleMigration {
    fn as_core_style(&self) -> impl CoreStyle + use<'_, Self>;
}

macro_rules! core_style_migrate {
    ($($fn_name:ident $ret:ty,)*) => {
        $(
            fn $fn_name(&self) -> $ret {
                self.as_core_style().$fn_name()
            }
        )*
    };
}

impl<T> CoreStyle for ResolvedStyle<'_, T>
where
    Self: CoreStyleMigration,
{
    core_style_migrate! {
        aspect_ratio             Option<f32>,
        border                   taffy::Rect<taffy::LengthPercentage>,
        box_generation_mode      taffy::BoxGenerationMode,
        box_sizing               taffy::BoxSizing,
        inset                    taffy::Rect<taffy::LengthPercentageAuto>,
        is_block                 bool,
        is_compressible_replaced bool,
        margin                   taffy::Rect<taffy::LengthPercentageAuto>,
        max_size                 taffy::Size<taffy::Dimension>,
        min_size                 taffy::Size<taffy::Dimension>,
        overflow                 taffy::Point<taffy::Overflow>,
        padding                  taffy::Rect<taffy::LengthPercentage>,
        position                 taffy::Position,
        scrollbar_width          f32,
        size                     taffy::Size<taffy::Dimension>,
    }
}
