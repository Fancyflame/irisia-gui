use cream_backend::WindowEvent;
use cream_core::{
    application::new_window,
    element::{Element, NeverInitalized, NoProps, RuntimeInit},
    event::{standard::ElementAbondoned, ElementEventKey, EventDispatcher},
    exit_app,
    primary::{Point, Region},
    read_style,
    render_fn,
    skia_safe::{Color, Color4f, Paint, Rect},
    structure::{StructureBuilder, VisitIter},
    style,
    winit::event::{ElementState, MouseButton},
    Event,
    Style, //Style,
};
use tokio::select;

#[cream_core::main]
async fn main() {
    new_window::<App, _>(|builder| builder.with_title("test"))
        .await
        .unwrap();
}

struct App {
    rects: Vec<Color>,
}

impl Element for App {
    type Props<'a> = NoProps;
    type ChildProps<'a> = NeverInitalized;

    fn create() -> Self {
        Self {
            rects: vec![Color::BLACK, Color::GREEN, Color::RED, Color::BLUE],
        }
    }

    render_fn! {
        @init(self);
        Flex {
            for (index, color) in self.rects.iter().enumerate() {
                @key index;
                Rectangle {
                    +listen: ("rect", index),
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
                    let (event, _) = init
                        .event_dispatcher
                        .recv::<WindowEvent, ElementEventKey>()
                        .await;

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
                let (_, key): (MyRequestClose, (&'static str, usize)) =
                    init.event_dispatcher.recv().await;
                println!("close request event received(sent by {:?})", key);
                init.close_handle.close();
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

#[derive(Style, Clone)]
#[cream(from)]
struct StyleColor(Color);

struct Rectangle {
    is_force: bool,
    force_color: Color,
}

impl Element for Rectangle {
    type Props<'a> = NoProps;
    type ChildProps<'a> = ();

    fn create() -> Self {
        Self {
            is_force: false,
            force_color: Color::CYAN,
        }
    }

    fn render<'r>(
        &mut self,
        _props: Self::Props<'_>,
        styles: &impl style::StyleContainer,
        region: Region,
        _cache_box_for_children: &mut cream_core::CacheBox,
        _event_dispatcher: &EventDispatcher,
        _children: cream_core::structure::Slot<
            impl StructureBuilder + cream_core::structure::VisitIter<Self::ChildProps<'r>>,
        >,
        mut content: cream_core::element::RenderContent,
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
                    let (window_event, _) =
                        init.window_event_dispatcher.recv::<WindowEvent, ()>().await;
                    match window_event {
                        WindowEvent::CloseRequested => {
                            println!("close event sent");
                            init.output_event_emitter.emit(&MyRequestClose);
                        }

                        _ => {}
                    }
                }
            };

            let b = async {
                loop {
                    select! {
                        _ = init.recv::<ElementAbondoned, ElementEventKey>() => {
                            println!("element dropped");
                            exit_app(0).await;
                            return;
                        },

                        (window_event,_) = init.recv::<WindowEvent, ElementEventKey>() => match window_event {
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

struct Flex;

impl Element for Flex {
    type Props<'a> = NoProps;
    type ChildProps<'a> = ();

    fn create() -> Self {
        Flex
    }

    fn render<'r>(
        &mut self,
        _props: Self::Props<'_>,
        _styles: &impl style::StyleContainer,
        drawing_region: Region,
        cache_box_for_children: &mut cream_core::CacheBox,
        _: &EventDispatcher,
        children: cream_core::structure::Slot<
            impl StructureBuilder + VisitIter<Self::ChildProps<'r>>,
        >,
        mut content: cream_core::element::RenderContent,
    ) -> cream_core::Result<()> {
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
