use irisia::{
    Result, Window, WinitWindow,
    application::PointerEvent,
    build2, coerce_hook,
    hook::Signal,
    model::{
        VModel, VNode,
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
        Corner, Point, Rect,
        length::{PCT, PX, VMIN},
        size::Size,
    },
    skia_safe::Color,
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

fn app() -> impl VNode<ParentProps = ()> {
    let red_rect = Signal::state(false);
    let switch_color: Signal<dyn Fn(PointerEvent)> = {
        let red_rect = red_rect.clone();
        coerce_hook!(
            Signal::state(move |pe: PointerEvent| {
                if let PointerEvent::PointerDown {
                    is_current: true,
                    position: _,
                } = pe
                {
                    let mut w = red_rect.write();
                    *w = !*w;
                }
            })
            .to_signal()
        )
    };

    let changing_rect = Signal::memo_ncmp(
        {
            let switch_color = switch_color.clone();
            move |&is_red| {
                build2! {
                    Block {
                        on := switch_color.clone(),
                        style: BlockStyle {
                            width: 0.5 * VMIN,
                            height: 0.5 * VMIN,
                            background: if is_red { Color::RED } else { Color::BLUE },
                            border_radius: Corner::all(50.0),
                            border_width: Rect {
                                left: 10 * PX,
                                ..Default::default()
                            },
                            border_color: Color::GRAY,
                            ..BlockStyle::DEFAULT
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

    //let text: Signal<dyn AsRef<str>> = coerce_hook!(text);

    build2! {
        CustomComp {
            vertical := red_rect.to_signal(),

            (changing_rect)

            Text {
                on := switch_color,
                // on PointerEvent(foo) => {

                // },
                text := coerce_hook!(text),
                style: TextStyle {
                    font_size: 40.0,
                    font_color: Color::MAGENTA,
                },

                // [foo]: 2, // emitted
            }

            for i in 0..2, key = i {
                Block {
                    style: BlockStyle {
                        width: 1 * PCT,
                        height: 10 * PX,
                        background: Color::BLACK,
                        border_radius: Corner::all(50.0),
                        ..BlockStyle::DEFAULT
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

#[derive(Default)]
struct CustomComp {
    vertical: Option<Signal<bool>>,
    children: Option<Signal<DynVModel<CustomProps>>>,
}

#[allow(unused)]
#[derive(Default, Debug, Clone)]
struct CustomProps {
    foo: i32,
}

impl Component for CustomComp {
    type ChildProps = CustomProps;
    fn create(
        self,
        _watcher_list: &mut irisia::hook::watcher::WatcherList,
    ) -> impl irisia::model::component::ComponentVNode {
        let styles = Signal::builder(Vec::new())
            .dep_call(
                |mut vec, styles| {
                    if let Some(styles) = styles {
                        vec.clear();
                        styles.get_parent_props(&mut |style| vec.push(style.clone()));
                    };
                },
                self.children.clone(),
                true,
            )
            .build();

        dbg!(&styles);

        let layout = Signal::memo(
            |vertical| AverageDivideLayout {
                vertical: vertical.copied().unwrap_or(false),
            },
            self.vertical,
        );

        build2! {
            Block {
                display := coerce_hook!(layout),
                style: BlockStyle {
                    width: 0.7 * PCT,
                    height: 0.7 * PCT,
                    ..Default::default()
                },
                (self.children.clear_parent_props())
            }
        }
    }
}

// Layouter implementation

#[derive(PartialEq, Eq)]
struct AverageDivideLayout {
    vertical: bool,
}

impl AverageDivideLayout {
    fn get_axis<T>(&self, size: Size<T>) -> (T, T)
    where
        T: Copy,
    {
        if self.vertical {
            (size.height, size.width)
        } else {
            (size.width, size.height)
        }
    }

    fn set_axis<T>(&self, main: T, sub: T) -> Size<T> {
        if self.vertical {
            Size {
                width: sub,
                height: main,
            }
        } else {
            Size {
                width: main,
                height: sub,
            }
        }
    }
}

impl BlockLayout for AverageDivideLayout {
    fn compute_layout(
        &self,
        children: LayoutChildren,
        constraint: Size<SpaceConstraint>,
    ) -> Size<f32> {
        let (main_axis_constraint, sub_axis_constraint) = self.get_axis(constraint);

        let main_axis_len = main_axis_constraint.get_numerical().unwrap_or(0.0);
        let sub_axis_len = sub_axis_constraint.get_numerical().unwrap_or(0.0);

        let main_axis_each_space = main_axis_len / children.len() as f32;
        //let mut sub_axis_len = 0f32;

        for (index, child) in children.iter().enumerate() {
            let size = child.measure(self.set_axis(
                SpaceConstraint::Exact(main_axis_each_space),
                SpaceConstraint::Exact(sub_axis_len),
            ));
            //sub_axis_len = sub_axis_len.max(self.get_axis(size).1);

            let location = self
                .set_axis(
                    index as f32 * main_axis_each_space,
                    (sub_axis_len - self.get_axis(size).1).max(0.0) / 2.0,
                )
                .to_point();

            child.set_location(location);
        }

        Size {
            width: main_axis_len,
            height: sub_axis_len,
        }
    }
}
