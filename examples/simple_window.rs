use std::time::Duration;

use irisia::{
    application::Window,
    build,
    data_flow::{
        register::{register, RcReg},
        Readable,
    },
    el_model::ElementAccess,
    element::{CompInputWatcher, ComponentTemplate, OneStructureCreate},
    event::standard::{PointerDown, PointerMove},
    skia_safe::Color,
    style,
    winit::window::WindowBuilder,
    Result,
};

use rand::Rng;
use window_backend::{Flex, Rectangle};

mod window_backend;

#[irisia::main]
async fn main() -> Result<()> {
    Window::new(
        WindowBuilder::new().with_title("hello irisia"),
        build!(App;),
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
    type Props<'a> = ();

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
                Color::MAGENTA,
                Color::BLACK,
            ]
            .into_iter()
            .map(|c| register(Some(c)))
            .collect::<Vec<_>>(),
        );

        let app = App { rects: vec };
        let children = app.structure();

        tokio::task::spawn_local({
            let rects = app.rects.clone();
            async move {
                loop {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    if rects.read().len() < 5 {
                        rects.write().push(register(Some(Color::RED)));
                    }
                }
            }
        });

        (app, children)
    }
}

impl App {
    fn structure(&self) -> impl OneStructureCreate {
        let rects = &self.rects;

        build! {
            input rects;

            Flex {
                for (index, item) in rects.iter().cloned().enumerate(),
                key = *index
                {
                    if let Some(color) = *item.read() {
                        reg force_height = (*index.read() == 2).then_some(30.0);
                        Rectangle {
                            force_color <= color.to_wire(),
                            force_height <= force_height.clone(),
                            @style: style! {
                                in window_backend::sty {
                                    Width: 50px;
                                    Height: 70px + 40px * *index.read() as f32;
                                }
                            },
                            @on_create: move |access| {
                                access.listen().spawn(move |_: PointerMove| {
                                    force_height.set(Some(rand::thread_rng().gen_range(50.0..200.0)));
                                });
                                access.listen().spawn(move |_: PointerDown| {
                                    rects.write().remove(*index.read());
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
