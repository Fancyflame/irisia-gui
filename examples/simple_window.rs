use irisia::{
    application::Window,
    build,
    data_flow::{
        register::{register, RcReg},
        Readable,
    },
    el_model::ElementAccess,
    element::{CompInputWatcher, Component, ComponentTemplate, EmptyProps, OneStructureCreate},
    event::standard::PointerDown,
    primitive::Length,
    skia_safe::Color,
    style,
    winit::window::WindowBuilder,
    Result,
};

use window_backend::{Flex, Rectangle};

mod window_backend;

#[irisia::main]
async fn main() -> Result<()> {
    Window::new(
        WindowBuilder::new().with_title("hello irisia"),
        build!(Component<App>;),
    )
    .await
    .unwrap()
    .join()
    .await;
}

struct App {
    rects: RcReg<Vec<RcReg<Option<Color>>>>,
}

impl ComponentTemplate for App {
    type Props<'a> = EmptyProps;

    fn create<Slt>(
        _: Self::Props<'_>,
        _: Slt,
        _: ElementAccess,
        _: CompInputWatcher<Self>,
    ) -> (Self, impl OneStructureCreate)
    where
        Slt: irisia::structure::StructureCreate,
    {
        let vec = register(
            [
                Color::RED,
                Color::YELLOW,
                Color::BLUE,
                Color::GREEN,
                Color::BLACK,
            ]
            .into_iter()
            .map(|c| register(Some(c)))
            .collect::<Vec<_>>(),
        );

        let app = App { rects: vec };
        let children = app.structure();

        (app, children)
    }
}

impl App {
    fn structure(&self) -> impl OneStructureCreate {
        let rects = &self.rects;
        build! {
            input rects;

            Flex {
                for (index, item) in rects.r().iter().cloned().enumerate(),
                key = *index
                {
                    if let Some(color) = *item.r().r() {
                        Rectangle {
                            force_color <= color.to_wire(),
                            @style: style! {
                                width: 100px;
                                height: 100px + 40px * index as f32;
                            },
                            @on_create: move |access| {
                                access.listen().spawn(move |_: PointerDown| {
                                    rects.write()[*index.r()].set(None);
                                });
                            },
                        }
                    }
                }
            }
        }
    }
}

/*impl ElementInterfaces for App {
    type BlankProps = ();

    fn set_children(&self, this: &RcElementModel<Self>) {
        this.set_children(build! {
            Flex {
                TextBox {
                    text: "Hello\nпpивeт\nこんにちは\n你好\n\nIrisia GUI🌺",
                    user_select: true,
                    +style: style! {
                        if 1 + 1 == 2 {
                            color: Color::MAGENTA;
                        }
                        font_weight: .bold;
                        font_size: 30px;
                    }
                }

                for (index, color) in self.rects.iter().enumerate() {
                    @key index;
                    if let Some(color) = color {
                        Rectangle {
                            +style: style! {
                                width: 100px;
                                height: 100px + 40px * index as f32;
                                color: *color;
                            },
                            +oncreate: move |em| {
                                rect_rt(this, em, index);
                            },
                        }
                    }
                }
            }
        })
        .layout_once(this.draw_region())
        .unwrap();
    }
}

impl ElementUpdateRaw<()> for App {
    fn el_create(this: &RcElementModel<Self>, _: ()) -> Self {
        this.global()
            .event_dispatcher()
            .listen()
            .no_handle()
            .spawn(|cr: CloseRequested| {
                println!("close requsted");
                cr.0.close();
            });

        Self {
            rects: vec![
                Some(Color::RED),
                Some(Color::YELLOW),
                Some(Color::BLUE),
                Some(Color::GREEN),
                Some(Color::BLACK),
            ],
        }
    }

    fn el_update(&mut self, _: &RcElementModel<Self>, _: (), _: bool) -> bool {
        true
    }
}

fn rect_rt(this: &ElModel!(App), rect: &ElModel!(Rectangle), index: usize) {
    println!("rectangle {index} got");
    let this = this.clone();
    rect.listen()
        .trusted()
        .no_handle()
        .once()
        .asyn()
        .spawn(move |_: PointerDown| async move {
            println!("rectangle {} deleted", index);
            this.el_mut().await.unwrap().rects[index].take();
        });
}*/
