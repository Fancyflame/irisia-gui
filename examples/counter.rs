use irisia::{
    Property, Result, Window, WinitWindow, build, coerce_hook,
    hook::{Signal, watcher::WatcherList},
    model::{
        VModel, VNode,
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
    JustifyContent,
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

    build! {
        Flex {
            style: FlexContainerStyle::DEFAULT
                .flex_direction(FlexDirection::Column)
                .justify_content(JustifyContent::Stretch)
                .align_content(AlignContent::Stretch),

            CenterBox {
                color: Color::BLUE,
                Text {
                    text[=]: Some(Signal::memo_ncmp(counter.to_read(), |count| {
                        format!("You clicked {count} times")
                    }).cast()),
                    style: TextStyle::DEFAULT
                        .font_size(30.0)
                        .font_color(Color::WHITE),
                }
            }

            CenterBox {
                color: Color::WHITE,
                // on: |event| {
                //     match event {

                //     }
                // },
            }
        }
    }
}

#[derive(Property)]
struct CenterBox {
    pub color: Signal<Color>,
    // pub children: Option<Signal<dyn CommonVModel<()>>>,
    #[prop(extend)]
    pub flex: Flex,
}

impl Component for CenterBox {
    fn create(self, _: &mut WatcherList) -> impl VNode<()> + use<> {
        build! {
            Flex {
                self: self.flex,
                style[=]: Some(Signal::memo_ncmp(self.color, |color| {
                    FlexContainerStyle::DEFAULT
                        .justify_content(JustifyContent::Center)
                        .align_items(AlignItems::Center)
                        .background(*color)
                })),
            }
        }
    }
}
