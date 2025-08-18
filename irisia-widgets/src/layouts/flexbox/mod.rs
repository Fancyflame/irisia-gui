use irisia::{
    build,
    hook::Signal,
    model::{component::Component, control_flow::CommonVModel, prim::Block},
    prim_element::block::BlockStyle,
    primitive::Length,
    style, Size,
};

pub use taffy::{AlignContent, FlexDirection, FlexWrap, JustifyContent};

use crate::layouts::{base_style::ChildStyle, flexbox::implement::FlexBlockLayout};

mod implement;

#[derive(Default)]
pub struct Flex {
    pub style: Option<Signal<FlexContainerStyle>>,
    pub children: Option<Signal<dyn CommonVModel<FlexItemStyle>>>,
}

impl Component for Flex {
    fn create(
        self,
        _watcher_list: &mut irisia::hook::watcher::WatcherList,
    ) -> impl irisia::model::VNode<()> + use<> {
        Signal::memo_ncmp(self.style, move |style| {
            let style = style.cloned().unwrap_or_default();
            build! {
                Block {
                    display: FlexBlockLayout {
                        container_style: style,
                    },
                    style: style.base,

                    // TODO: 避免多一次signal
                    (self.children.clone())
                }
            }
        })
    }
}

#[style(FlexContainerStyleExt)]
#[derive(Clone, Copy, PartialEq)]
pub struct FlexContainerStyle {
    pub flex_direction: FlexDirection,
    pub flex_wrap: FlexWrap,
    pub gap: Size<Length>,
    pub align_content: AlignContent,
    pub align_items: AlignItems,
    pub justify_content: JustifyContent,

    #[style(extend)]
    pub base: BlockStyle,
}

impl FlexContainerStyle {
    pub const DEFAULT: Self = Self {
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::Wrap,
        gap: Size::all(Length::Auto),
        align_content: AlignContent::FlexStart,
        align_items: AlignItems::Auto,
        justify_content: JustifyContent::FlexStart,
        base: BlockStyle::DEFAULT,
    };
}

impl Default for FlexContainerStyle {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[style(FlexItemStyleExt)]
#[derive(Clone, Copy, PartialEq)]
pub struct FlexItemStyle {
    pub flex_basis: Length,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub align_self: AlignSelf,

    #[style(extend)]
    pub base: ChildStyle,
}

impl FlexItemStyle {
    pub const DEFAULT: Self = Self {
        flex_basis: Length::Auto,
        flex_grow: 0.0,
        flex_shrink: 1.0,
        align_self: AlignSelf::Auto,
        base: ChildStyle::DEFAULT,
    };
}

impl Default for FlexItemStyle {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AlignItems {
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
    Stretch,
    Auto,
}

impl AlignItems {
    fn to_taffy(self) -> Option<taffy::AlignItems> {
        macro_rules! map {
            ($($Var:ident,)*) => {
                match self {
                    Self::Auto => None,
                    $(Self::$Var => Some(taffy::AlignItems::$Var),)*
                }
            };
        }
        map![Start, End, FlexStart, FlexEnd, Center, Baseline, Stretch,]
    }
}

pub type AlignSelf = AlignItems;
