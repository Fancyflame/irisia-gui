use cream_backend::{start_runtime, WindowEvent};
use cream_core::{
    application::new_window,
    element::{Element, NoProps, RuntimeInit},
    event::standard::ElementDropped,
    exit_app, match_event,
    primary::{Point, Region},
    read_style, render_fn, select_event,
    skia_safe::{Color, Color4f, Paint, Rect},
    structure::Node,
    style,
    winit::event::{ElementState, MouseButton},
    Event, Result,
};
use cream_macros::Style;

fn main() {
    start_runtime(async {
        new_window::<App, _>(|builder| builder.with_title("test"))
            .await
            .unwrap();
    });
}

struct App(u32);

impl Element for App {
    type Props<'a> = NoProps;
    fn create() -> Self {
        Self(10)
    }

    render_fn! {
        @init(self);
        Rectangle {
            +style: style!{
                width: 570.0;
                height: 440.0;
                if self.0 != 2 {
                    color: Color::RED;
                } else {
                    color: Color::BLUE;
                }
            }
        }
    }

    fn start_runtime(init: RuntimeInit<Self>) {
        tokio::spawn(async move {
            let element = init.get_receiver("@element").await;
            loop {
                match_event! {
                    element.recv().await => {
                        window_event as WindowEvent => match window_event{
                            WindowEvent::MouseInput{ button: MouseButton::Left, state: ElementState::Pressed, ..}=>{
                                println!("left click");
                            },
                            _=>{}
                        }
                    }
                }
            }
        });
    }
}

#[derive(Style, Clone)]
#[cream(from)]
struct StyleWidth(f32);

#[derive(Style, Clone)]
#[cream(from)]
struct StyleHeight(f32);

#[derive(Style, Clone)]
#[cream(from)]
struct StyleColor(Color);

struct Rectangle {
    is_force: bool,
    force_color: Color,
}

impl Element for Rectangle {
    type Props<'a> = NoProps;

    fn create() -> Self {
        Self {
            is_force: false,
            force_color: Color::CYAN,
        }
    }

    fn render(
        &mut self,
        _props: Self::Props<'_>,
        styles: &impl style::StyleContainer,
        _cache_box: &mut cream_core::CacheBox,
        _chan_setter: &cream_core::event::EventChanSetter,
        _children: cream_core::structure::Slot<impl Node>,
        mut content: cream_core::element::RenderContent,
        region_requester: &mut dyn FnMut(Option<Region>) -> Result<Region>,
    ) -> cream_core::Result<()> {
        read_style!(styles => {
            w: Option<StyleWidth>,
            h: Option<StyleHeight>,
            c: Option<StyleColor>,
        });

        let (w, h) = (
            w.unwrap_or(StyleWidth(50.0)),
            h.unwrap_or(StyleHeight(50.0)),
        );

        let region = region_requester(None)?;

        content.set_interact_region((region.0, region.0 + Point(w.0 as _, h.0 as _)));

        let rect = Rect::new(0.0, 0.0, w.0, h.0);
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
            let element = init.get_receiver("@element").await;
            let window = init.get_receiver("@window").await;
            loop {
                select_event! {
                    element.recv() => {
                        _ as ElementDropped => {
                            println!("element dropped");
                            exit_app(0).await;
                            return;
                        }

                        window_event as WindowEvent => match window_event {
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

                        _ as MyEvent => {},
                    },

                    window.recv()=>{
                        window_event as WindowEvent => match window_event {
                            WindowEvent::CloseRequested => {
                                println!("close request");
                                init.close_handle.close();
                            },

                            _ => {}
                        },
                    },
                };
            }
        });
    }
}

#[derive(Event)]
pub struct MyEvent;
