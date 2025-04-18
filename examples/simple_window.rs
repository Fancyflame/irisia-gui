use irisia::{
    application::PointerEvent,
    build2, coerce_hook,
    hook::{reactive::Reactive, Signal},
    model::{
        component::Component,
        control_flow::CommonVModel,
        prim::{Block, Rect, Text},
        VNode,
    },
    prim_element::{block::LayoutFn, rect::RectStyle, text::TextStyle, Element},
    primitive::{Point, Region},
    skia_safe::Color,
    Result, Window, WinitWindow,
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

fn app() -> impl VNode {
    let red_rect = Signal::state(false);
    let switch_color: Signal<dyn Fn(PointerEvent)> = {
        let red_rect = red_rect.clone();
        coerce_hook!(Signal::state(move |pe: PointerEvent| {
            if let PointerEvent::PointerDown {
                is_current: true,
                position: _,
            } = pe
            {
                let mut w = red_rect.write();
                *w = !*w;
            }
        })
        .to_signal())
    };

    let changing_rect = Signal::memo_ncmp(
        {
            let switch_color = switch_color.clone();
            move |&is_red| {
                build2! {
                    Rect {
                        on := switch_color.clone(),
                        style: RectStyle {
                            color: if is_red { Color::RED } else { Color::BLUE },
                            border_radius: [50.0; 4],
                        },
                    }
                }
            }
        },
        red_rect.to_signal(),
    );
    let text = Signal::memo_ncmp(
        |&is_red| {
            format!(
                "点击该文本或{0}色矩形切换颜色：当前显示{0}色",
                if is_red { "红" } else { "蓝" }
            )
        },
        red_rect.to_signal(),
    );

    build2! {
        CustomComp {
            (changing_rect) in {
                [foo]: 1,
            }

            Text {
                on := switch_color,
                // on PointerEvent(foo) => {

                // },
                text := text,
                style: TextStyle {
                    font_size: 40.0,
                    font_color: Color::MAGENTA,
                },
                // [foo]: 2, // emitted
            }

            for i in 0..2, key = i {
                Rect {
                    style: RectStyle {
                        color: Color::BLACK,
                        border_radius: [50.0; 4],
                    },
                    [foo]: 3,
                }
            }

            match 10 {
                1 => (),
                2 => (),
                _ => Text {
                    style: TextStyle {
                        font_color: Color::BLACK,
                        font_size: 20.0,
                    },
                    text: "match表达式".to_string(),
                    [foo]: 4,
                },
            }
        }
    }
}

fn average_divide(size: Point, el: &[Element], regions: &mut Vec<Region>) {
    let width = size.0 / el.len() as f32;
    let height = size.1;

    for i in 0..el.len() {
        let i = i as f32;
        regions.push(Region::new(
            Point(i * width, 0.0),
            Point((i + 1.0) * width, height),
        ));
    }
}

#[derive(Default)]
struct CustomComp {
    children: Option<Signal<dyn CommonVModel>>,
    children_props: Option<Signal<Vec<CustomProps>>>,
}

#[allow(unused)]
#[derive(Default, Debug, Clone)]
struct CustomProps {
    foo: i32,
}

impl Component for CustomComp {
    type ChildProps = CustomProps;
    type Created = ();
    fn create(self) -> (Self::Created, impl VNode) {
        dbg!(&self.children_props);

        (
            (),
            build2! {
                Block {
                    layout_fn: average_divide as LayoutFn,
                    (self.children)
                }
            },
        )
    }
}
