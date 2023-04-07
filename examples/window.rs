use cream::{
    application::Window,
    element::{Element, Frame, NeverInitalized, NoProps, RuntimeInit},
    event::standard::ElementAbondoned,
    exit_app,
    primary::Point,
    read_style, render_fn,
    skia_safe::{Color, Color4f, Paint, Rect},
    structure::{StructureBuilder, VisitIter},
    style,
    style::StyleColor,
    textbox::{styles::*, TextBox},
    winit::event::{ElementState, MouseButton},
    Event, Style, WindowEvent,
};
use tokio::select;

#[cream::main]
async fn main() {
    Window::new::<App>("test".into())
        .await
        .unwrap()
        .join()
        .await;
}

struct App {
    rects: Vec<Color>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            rects: vec![
                Color::RED,
                Color::YELLOW,
                Color::BLUE,
                Color::GREEN,
                Color::BLACK,
            ],
        }
    }
}

impl Element for App {
    type Props<'a> = NoProps;
    type ChildProps<'a> = NeverInitalized;

    render_fn! {
        @init(self);
        Flex {
            TextBox {
                text: "hello世界🌏",
                +style: style!{
                    color: Color::MAGENTA;
                    font_slant: .normal;
                    font_size: 50px;
                }
            }

            for (index, color) in self.rects.iter().enumerate() {
                @key index;
                Rectangle {
                    +id: ("rect", index),
                    +style: style!{
                        width: 100.0;
                        height: 100.0 + 40.0 * index as f32;
                        color: color.clone();
                    }
                }
            }
        }
    }

    fn start_runtime(init: RuntimeInit<Self>) {
        tokio::spawn(async move {
            let a = async {
                loop {
                    let event = init.event_dispatcher.recv_sys::<WindowEvent>().await;

                    match event {
                        WindowEvent::MouseInput {
                            button: MouseButton::Left,
                            state: ElementState::Pressed,
                            ..
                        } => {
                            println!("left click");
                        }
                        _ => {}
                    }
                }
            };

            let b = async {
                loop {
                    let rect = init
                        .event_dispatcher
                        .get_element_checked(|(s, _): &(&str, usize)| *s == "rect")
                        .await;

                    tokio::spawn(async move {
                        println!("recv{} got", rect.key.1);

                        rect.result.recv::<MyRequestClose>().await;
                        println!("close request event received(sent by {:?})", rect.key.1);
                        init.close_handle.close();
                    });
                }
            };

            tokio::join!(a, b);
        });
    }
}

#[derive(Style, Clone)]
#[cream(from)]
struct StyleWidth(f32);

#[derive(Style, Clone)]
#[cream(from)]
struct StyleHeight(f32);

struct Rectangle {
    is_force: bool,
    force_color: Color,
}

impl Default for Rectangle {
    fn default() -> Self {
        Self {
            is_force: false,
            force_color: Color::CYAN,
        }
    }
}

impl Element for Rectangle {
    type Props<'a> = NoProps;
    type ChildProps<'a> = ();

    fn render<'a>(
        &mut self,
        Frame {
            styles,
            drawing_region: region,
            mut content,
            ..
        }: cream::element::Frame<
            Self,
            impl style::StyleContainer,
            impl VisitIter<Self::ChildProps<'a>>,
        >,
    ) -> cream::Result<()> {
        read_style!(styles => {
            w: Option<StyleWidth>,
            h: Option<StyleHeight>,
            c: Option<StyleColor>,
        });

        let (w, h) = (
            w.unwrap_or(StyleWidth(50.0)),
            h.unwrap_or(StyleHeight(50.0)),
        );

        content.set_interact_region((region.0, region.0 + Point(w.0 as _, h.0 as _)));

        let rect = Rect::new(
            region.0 .0 as _,
            region.0 .1 as _,
            region.0 .0 as f32 + w.0,
            region.0 .1 as f32 + h.0,
        );
        let color = if self.is_force {
            self.force_color.clone()
        } else {
            c.unwrap_or(StyleColor(Color::GREEN)).0
        };

        let paint = Paint::new(Color4f::from(color), None);

        content.canvas().draw_rect(rect, &paint);

        Ok(())
    }

    fn start_runtime(init: RuntimeInit<Self>) {
        tokio::spawn(async move {
            let a = async {
                loop {
                    let window_event = init.window_event_dispatcher.recv_sys::<WindowEvent>().await;
                    match window_event {
                        WindowEvent::CloseRequested => {
                            println!("close event sent");
                            init.event_dispatcher.emit(MyRequestClose);
                        }

                        _ => {}
                    }
                }
            };

            let b = async {
                loop {
                    select! {
                        _ = init.recv_sys::<ElementAbondoned>() => {
                            println!("element dropped");
                            exit_app(0).await;
                            return;
                        },

                        window_event = init.recv_sys::<WindowEvent>() => match window_event {
                            WindowEvent::MouseInput {
                                state,
                                ..
                            } => {
                                init.app.lock().await.is_force = match state {
                                    ElementState::Pressed => true,
                                    ElementState::Released => false,
                                }
                            },

                            _ => {}
                        },
                    }
                }
            };

            tokio::join!(a, b);
        });
    }
}

#[derive(Event, Clone)]
pub struct MyRequestClose;

#[derive(Default)]
struct Flex;

impl Element for Flex {
    type Props<'a> = NoProps;
    type ChildProps<'a> = ();

    fn render<'a>(
        &mut self,
        Frame {
            drawing_region,
            cache_box_for_children,
            mut content,
            children,
            ..
        }: Frame<Self, impl style::StyleContainer, impl VisitIter<Self::ChildProps<'a>>>,
    ) -> cream::Result<()> {
        let (start, end) = drawing_region;
        let abs = end - start;

        let rendering = children.into_rendering(cache_box_for_children, content.inherit());
        let len = rendering.children_count();
        let width = abs.0 / len as u32;

        let mut index = 0;
        rendering.finish_iter(|(), _| {
            let result = Ok((
                Point(index * width, start.1),
                Point((index + 1) * width, end.1),
            ));
            index += 1;
            result
        })
    }
}
