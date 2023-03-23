use cream_backend::{start_runtime, window_handle::close_handle::CloseHandle, WindowEvent};
use cream_core::{
    application::new_window,
    element::{Element, NoProps},
    event::standard::ElementDropped,
    exit_app, match_event, read_style, render_fn, select_event,
    skia_safe::{Color, Color4f, Paint, Rect},
    structure::{EmptyStructure, Node},
    style,
    winit::event::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode},
    Event,
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
    type Children<Ch: Node> = EmptyStructure;
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

    fn start_runtime(
        _slf: std::sync::Arc<tokio::sync::Mutex<Self>>,
        _event_emitter: cream_core::event::EventEmitter,
        chan_getter: cream_core::event::EventChanGetter,
        _close_handle: CloseHandle,
    ) {
        tokio::spawn(async move {
            let global = chan_getter.get_receiver("@global").await;
            loop {
                match_event! {
                    global.recv().await => {
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
    type Children<Ch: Node> = EmptyStructure;
    type Props<'a> = NoProps;

    fn create() -> Self {
        Self {
            is_force: false,
            force_color: Color::CYAN,
        }
    }

    fn render<S, C>(
        &mut self,
        _props: Self::Props<'_>,
        styles: &S,
        _chan_setter: &cream_core::event::EventChanSetter,
        _cache_box: &mut cream_core::CacheBox,
        _children: cream_core::structure::Slot<C>,
        mut content: cream_core::element::RenderContent,
    ) -> cream_core::Result<()>
    where
        S: cream_core::style::StyleContainer,
        C: Node,
        Self: Element<Children<C> = C>,
    {
        read_style!(styles => {
            w: Option<StyleWidth>,
            h: Option<StyleHeight>,
            c: Option<StyleColor>,
        });

        let (w, h) = (
            w.unwrap_or(StyleWidth(50.0)),
            h.unwrap_or(StyleHeight(50.0)),
        );

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

    fn start_runtime(
        slf: std::sync::Arc<tokio::sync::Mutex<Self>>,
        _event_emitter: cream_core::event::EventEmitter,
        chan_getter: cream_core::event::EventChanGetter,
        close_handle: CloseHandle,
    ) {
        tokio::spawn(async move {
            let receiver = chan_getter.get_receiver("@window").await;
            let element = chan_getter.get_receiver("@element").await;
            loop {
                select_event! {
                    element.recv() => {
                        _ as ElementDropped => {
                            println!("element dropped");
                            exit_app(0).await;
                            return;
                        }
                    },

                    receiver.recv() => {
                        window_event as WindowEvent, _ as () => match window_event {
                            WindowEvent::KeyboardInput {
                                input:
                                    KeyboardInput {
                                        state,
                                        virtual_keycode: Some(VirtualKeyCode::Space),
                                        ..
                                    },
                                ..
                            } => {
                                slf.lock().await.is_force = match state {
                                    ElementState::Pressed => true,
                                    ElementState::Released => false,
                                }
                            },

                            WindowEvent::CloseRequested => {
                                close_handle.close();
                            },

                            _ => {}
                        },

                        _ as MyEvent => {},
                    },
                };
            }
        });
    }
}

#[derive(Event)]
pub struct MyEvent;
