use irisia::{
    element::{Element, Frame, InitContent, NoProps},
    event::standard::{ElementAbandoned, PointerEntered, PointerOut},
    exit_app,
    primitive::Point,
    read_style,
    skia_safe::{Color, Color4f, Paint, Rect},
    structure::{StructureBuilder, VisitIter},
    style,
    style::StyleColor,
    Event, StaticWindowEvent, Style,
};
use tokio::select;

#[derive(Style, Clone)]
#[irisia(style(from))]
pub struct StyleWidth(f32);

#[derive(Style, Clone)]
#[irisia(style(from))]
pub struct StyleHeight(f32);

pub struct Rectangle {
    is_force: bool,
    force_color: Color,
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
        }: irisia::element::Frame<
            Self,
            impl style::StyleContainer,
            impl VisitIter<Self::ChildProps<'a>>,
        >,
    ) -> irisia::Result<()> {
        read_style!(styles => {
            w: Option<StyleWidth>,
            h: Option<StyleHeight>,
            c: Option<StyleColor>,
        });

        let (w, h) = (
            w.unwrap_or(StyleWidth(50.0)),
            h.unwrap_or(StyleHeight(50.0)),
        );

        content.set_interact_region((region.0, region.0 + Point(w.0, h.0)));

        let rect = Rect::new(
            region.0 .0,
            region.0 .1,
            region.0 .0 + w.0,
            region.0 .1 + h.0,
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

    fn create(init: &InitContent<Self>) -> Self {
        let init = init.clone();
        tokio::spawn(async move {
            let app = init.app.upgrade().unwrap();

            let a = async {
                loop {
                    let window_event = init
                        .window_event_dispatcher
                        .recv_sys::<StaticWindowEvent>()
                        .await;
                    match window_event {
                        StaticWindowEvent::CloseRequested => {
                            println!("close event sent");
                            init.event_handle.emit(MyRequestClose);
                        }

                        _ => {}
                    }
                }
            };

            let b = async {
                loop {
                    select! {
                        _ = init.recv_sys::<ElementAbandoned>() => {
                            println!("element dropped");
                            exit_app(0).await;
                            return;
                        },

                        _=init.recv_sys::<PointerEntered>()=>{
                            app.lock().await.is_force=true;
                        }

                        _=init.recv_sys::<PointerOut>()=>{
                            app.lock().await.is_force=false;
                        }
                    }
                }
            };

            let c = async {
                loop {
                    select! {
                        _ = init.recv_sys::<PointerEntered>() => {
                            println!("pointer entered");
                        },

                        _ = init.recv_sys::<PointerOut>() => {
                            println!("pointer out");
                        },
                    }
                }
            };

            tokio::join!(a, b, c);
        });

        Self {
            is_force: false,
            force_color: Color::CYAN,
        }
    }
}

#[derive(Event, Clone)]
pub struct MyRequestClose;

pub struct Flex;

impl Element for Flex {
    type Props<'a> = NoProps;
    type ChildProps<'a> = ();

    fn create(_: &InitContent<Self>) -> Self {
        Flex
    }

    fn render<'a>(
        &mut self,
        Frame {
            drawing_region,
            mut content,
            children,
            ..
        }: Frame<Self, impl style::StyleContainer, impl VisitIter<Self::ChildProps<'a>>>,
    ) -> irisia::Result<()> {
        let (start, end) = drawing_region;
        let abs = end - start;

        let rendering = children.into_rendering(&mut content);
        let len = rendering.children_count();
        let width = abs.0 / len as f32;

        let mut index = 0;
        rendering.finish_iter(|(), _| {
            let result = Ok((
                Point(index as f32 * width, start.1),
                Point((index as f32 + 1.0) * width, end.1),
            ));
            index += 1;
            result
        })
    }
}
