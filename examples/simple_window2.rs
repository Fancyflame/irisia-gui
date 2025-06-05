use std::cell::Cell;

use irisia::{
    Result, Window, WinitWindow,
    application::PointerEvent,
    build2, coerce_hook,
    hook::Signal,
    model::{
        VNode,
        component::Component,
        control_flow::common_vmodel::DynVModel,
        prim::{Block, Text},
    },
    prim_element::{
        block::{BlockLayout, BlockStyle, layout::LayoutChildren},
        layout::SpaceConstraint,
        text::TextStyle,
    },
    primitive::{
        Corner, Rect,
        length::{PCT, PX, VH, VMIN, VW},
        size::Size,
    },
    skia_safe::Color,
};
use irisia_widgets::layouts::{
    AlignContent, AlignItems, FlexContainerStyle, FlexDirection, FlexItemStyle, JustifyContent,
    base_style::ChildStyle, flexbox::Flex,
};

#[irisia::main]
async fn main() -> Result<()> {
    Window::new(
        WinitWindow::default_attributes().with_title("hello irisia"),
        app,
    )
    .await
    .unwrap()
    .join()
    .await;
}

fn app() -> impl VNode<()> {
    let flex_direction = Signal::state(FlexDirection::Row);

    let is_row_cell = Cell::new(true);
    let extra_blocks = Signal::state(Vec::new());

    let on_click = {
        let flex_direction = flex_direction.clone();
        let extra_blocks = extra_blocks.clone();
        move |pe: PointerEvent| {
            if let PointerEvent::PointerDown {
                is_current: true, ..
            } = pe
            {
                let is_row = !is_row_cell.get();
                is_row_cell.set(is_row);
                flex_direction.set(if is_row {
                    FlexDirection::Row
                } else {
                    FlexDirection::Column
                });

                let mut eb = extra_blocks.write();
                let eb_len = eb.len();
                eb.push((eb_len, create_block()));
            }
        }
    };

    build2! {
        Flex {
            style := Signal::memo(flex_direction.to_signal(), |&flex_direction| FlexContainerStyle {
                flex_direction,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                align_content: AlignContent::Center,
                base: BlockStyle {
                    background: Color::GRAY,
                    ..Default::default()
                },
                ..Default::default()
            }),

            Block::<()> {
                style: BlockStyle {
                    background: Color::RED,
                    ..Default::default()
                },
                super: FlexItemStyle {
                    base: ChildStyle {
                        width: 60 * PX,
                        height: 30 * PX,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            }

            Block::<()> {
                style: BlockStyle {
                    background: Color::BLACK,
                    ..Default::default()
                },
                on: on_click,
                super: FlexItemStyle {
                    base: ChildStyle {
                        width: 0.2 * PCT,
                        height: 0.3 * VH,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            }

            Block::<()> {
                style: BlockStyle {
                    background: Color::BLUE,
                    ..Default::default()
                },
                super: FlexItemStyle {
                    base: ChildStyle {
                        width: 0.2 * VW,
                        height: 0.1 * PCT,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            }

            (extra_blocks.to_signal())
        }
    }
}

fn create_block() -> impl VNode<FlexItemStyle> {
    build2! {
        Block::<()> {
            style: BlockStyle {
                background: Color::RED,
                ..Default::default()
            },
            super: FlexItemStyle {
                base: ChildStyle {
                    width: 60 * PX,
                    height: 30 * PX,
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }
}
