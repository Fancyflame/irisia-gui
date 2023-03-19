use cream_backend::{start_runtime, WindowEvent};
use cream_core::{
    application::run_application,
    element::{Element, NoProps},
    primary::Point,
    skia_safe::{Color, Color4f, ColorSpace, Paint, Rect},
    structure::{EmptyStructure, Node},
    style,
    winit::{
        dpi::PhysicalSize,
        event::{ElementState, KeyboardInput, VirtualKeyCode},
    },
};
use cream_macros::Style;

fn main() {
    start_runtime(|| {
        run_application::<App, _>(|w| {
            w.with_inner_size(PhysicalSize {
                width: 1000,
                height: 800,
            })
            .with_title("test")
        })
        .unwrap()
    });
}

struct App;

impl Element for App {
    type Children<Ch: Node> = EmptyStructure;
    type Props<'a> = NoProps;
    fn create() -> Self {
        Self
    }

    fn render<S, C>(
        &mut self,
        props: Self::Props<'_>,
        styles: &S,
        cache_box: &mut cream_core::CacheBox,
        children: cream_core::structure::Slot<C>,
        chan_setter: &cream_core::event::EventChanSetter,
        content: cream_core::element::RenderContent,
    ) -> cream_core::Result<()>
    where
        S: cream_core::style::StyleContainer,
        C: Node,
        Self: Element<Children<C> = C>,
    {
        cream_core::render! {
            @init(chan_setter, cache_box, content);
            Rectangle {
                +style: style!{
                    width: 570.0;
                    height: 440.0;
                    color: Color::RED;
                }
            }
        }
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
        _cache_box: &mut cream_core::CacheBox,
        _children: cream_core::structure::Slot<C>,
        _chan_setter: &cream_core::event::EventChanSetter,
        mut content: cream_core::element::RenderContent,
    ) -> cream_core::Result<()>
    where
        S: cream_core::style::StyleContainer,
        C: Node,
        Self: Element<Children<C> = C>,
    {
        let (w, h, c) =
            styles.read::<(Option<StyleWidth>, Option<StyleHeight>, Option<StyleColor>)>();
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
    ) {
        let closure = async move {
            let receiver = chan_getter.get_receiver("@global").await;
            loop {
                let data = receiver.recv().await;
                if let WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state,
                            virtual_keycode: Some(VirtualKeyCode::Space),
                            ..
                        },
                    ..
                } = data.assume::<WindowEvent>().expect("no window event")
                {
                    slf.lock().await.is_force = match state {
                        ElementState::Pressed => true,
                        ElementState::Released => false,
                    }
                }
            }
        };
        cream_backend::TOKIO_RT.spawn(closure);
    }
}
