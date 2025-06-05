use irisia::{
    prim_element::block::{layout::LayoutChildren, BlockLayout, BlockStyle},
    primitive::{length::LengthStandard, Length},
    Size,
};
use taffy::{
    prelude::FromLength, AlignContent, AlignItems, AlignSelf, FlexDirection, FlexWrap,
    JustifyContent, LayoutFlexboxContainer,
};

use crate::layouts::{
    assert_parent_node, base_style::ChildStyle, local_layout_input_to_taffy, size_to_local,
    size_to_taffy, CoreStyleMigration, MyTree, ResolvedStyle, PARENT_NODE_ID,
};

use super::{FlexContainerStyle, FlexItemStyle};

#[derive(Clone)]
pub(super) struct FlexBlockLayout {
    pub container_style: FlexContainerStyle,
}

impl BlockLayout<FlexItemStyle> for FlexBlockLayout {
    fn compute_layout(
        &self,
        children: LayoutChildren<FlexItemStyle>,
        constraints: Size<irisia::prim_element::layout::SpaceConstraint>,
    ) -> Size<f32> {
        let parent_size = children.parent_size().map(Some);
        size_to_local(
            taffy::compute_flexbox_layout(
                &mut MyTree {
                    container_style: &self.container_style,
                    children,
                },
                PARENT_NODE_ID,
                local_layout_input_to_taffy(constraints, parent_size),
            )
            .size,
        )
    }
}

impl<'tree> LayoutFlexboxContainer for MyTree<'tree, FlexContainerStyle, FlexItemStyle> {
    type FlexboxContainerStyle<'a>
        = ResolvedStyle<'a, &'a FlexContainerStyle>
    where
        Self: 'a;

    type FlexboxItemStyle<'a>
        = ResolvedStyle<'a, AxisFlexItemStyle<'a>>
    where
        Self: 'a;

    fn get_flexbox_child_style(&self, child_node_id: taffy::NodeId) -> Self::FlexboxItemStyle<'_> {
        let is_vertical = matches!(
            self.container_style.flex_direction,
            FlexDirection::Column | FlexDirection::ColumnReverse
        );

        ResolvedStyle {
            length_standard: self.children.length_standard().as_ref(),
            style: AxisFlexItemStyle {
                style: self.children.get(child_node_id.into()).data(),
                is_vertical,
            },
        }
    }

    fn get_flexbox_container_style(
        &self,
        node_id: taffy::NodeId,
    ) -> Self::FlexboxContainerStyle<'_> {
        assert_parent_node(node_id);
        ResolvedStyle {
            length_standard: self.children.length_standard().as_ref(),
            style: &self.container_style,
        }
    }
}

impl taffy::CoreStyle for ResolvedStyle<'_, &FlexContainerStyle> {
    // everything use default
}

impl taffy::FlexboxContainerStyle for ResolvedStyle<'_, &FlexContainerStyle> {
    fn align_content(&self) -> Option<AlignContent> {
        Some(self.style.align_content)
    }

    fn align_items(&self) -> Option<AlignItems> {
        Some(self.style.align_items)
    }

    fn flex_direction(&self) -> FlexDirection {
        self.style.flex_direction
    }

    fn flex_wrap(&self) -> FlexWrap {
        self.style.flex_wrap
    }

    fn gap(&self) -> taffy::Size<taffy::LengthPercentage> {
        size_to_taffy(self.style.gap.map_with(self.length_standard, |x, ls| {
            taffy::LengthPercentage::length(ls.resolve(x).unwrap_or(0.0))
        }))
    }

    fn justify_content(&self) -> Option<JustifyContent> {
        Some(self.style.justify_content)
    }
}

pub(crate) struct AxisFlexItemStyle<'a> {
    style: &'a FlexItemStyle,
    is_vertical: bool,
}

impl<'a, 'b> CoreStyleMigration for ResolvedStyle<'a, AxisFlexItemStyle<'b>> {
    fn as_core_style(&self) -> impl taffy::CoreStyle + use<'_, 'a, 'b> {
        self.wrap(&self.style.style.base)
    }
}

impl taffy::FlexboxItemStyle for ResolvedStyle<'_, AxisFlexItemStyle<'_>> {
    fn align_self(&self) -> Option<AlignSelf> {
        Some(self.style.style.align_self)
    }

    fn flex_basis(&self) -> taffy::Dimension {
        let ls = if self.style.is_vertical {
            self.length_standard.height
        } else {
            self.length_standard.width
        };

        match ls.resolve(self.style.style.flex_basis) {
            Some(value) => taffy::Dimension::length(value),
            None => taffy::Dimension::auto(),
        }
    }

    fn flex_grow(&self) -> f32 {
        self.style.style.flex_grow
    }

    fn flex_shrink(&self) -> f32 {
        self.style.style.flex_grow
    }
}
