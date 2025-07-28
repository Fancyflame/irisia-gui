use irisia::{
    Result, Window, WinitWindow, build2,
    hook::{Signal, watcher::WatcherList},
    model::{
        VNode,
        component::Component,
        control_flow::CommonVModel,
        prim::{Block, Text},
    },
    prim_element::{
        block::BlockStyleExt,
        text::{TextStyle, TextStyleExt},
    },
    skia_safe::Color,
};
use irisia_widgets::layouts::{
    AlignContent, AlignItems, Flex, FlexContainerStyle, FlexContainerStyleExt, FlexDirection,
    FlexItemStyle, JustifyContent,
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
    let counter = Signal::state(0);

    build2! {
        Flex {
            style: FlexContainerStyle::DEFAULT
                .flex_direction(FlexDirection::Column)
                .justify_content(JustifyContent::Stretch)
                .align_content(AlignContent::Stretch),

            CenterBox {
                color: Color::BLUE,
                Text {
                    text := Signal::memo_ncmp(counter.to_signal(), |count| {
                        format!("You clicked {count} times")
                    }).into(),
                    style: TextStyle::DEFAULT
                        .font_size(20.0)
                        .font_color(Color::WHITE),
                }
            }

            CenterBox {
                color: Color::WHITE,
                on: |event| {
                    match event {

                    }
                },
            }
        }
    }
}

#[derive(Default)]
struct CenterBox {
    pub color: Option<Signal<Color>>,
    pub children: Option<Signal<dyn CommonVModel<FlexItemStyle>>>,
}

impl Component for CenterBox {
    fn create(self, _: &mut WatcherList) -> impl VNode<()> + use<> {
        build2! {
            Flex {
                style := Signal::memo_ncmp(self.color, |color| {
                    FlexContainerStyle::DEFAULT
                        .justify_content(JustifyContent::Center)
                        .align_items(AlignItems::Center)
                        .background(color.copied().unwrap())
                }),

                (self.children)
            }
        }
    }
}
