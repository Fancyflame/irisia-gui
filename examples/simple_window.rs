use std::rc::Rc;

use irisia::{
    application::Window,
    data_flow::{
        register::{register, Register},
        wire, Readable,
    },
    element::{Component, ComponentTemplate, ElementInterfaces},
    event::standard::{CloseRequested, PointerDown},
    skia_safe::Color,
    structure::{branch, repeat, single},
    winit::window::WindowBuilder,
    Result,
};

use window_backend::{Flex, RectProps, Rectangle, StyleHeight, StyleWidth};

mod window_backend;

#[irisia::main]
async fn main() -> Result<()> {
    Window::new(
        WindowBuilder::new().with_title("hello irisia"),
        single::<Component<App>, _>((), (), (), |_| {}),
    )
    .await
    .unwrap()
    .join()
    .await;
}

struct App {
    rects: Rc<Register<Vec<Rc<Register<Option<Color>>>>>>,
}

impl ComponentTemplate for App {
    type Props<'a> = ();

    fn create<Slt>(
        _: Self::Props<'_>,
        _: Slt,
        _: irisia::el_model::ElementAccess,
        _: irisia::element::CompInputWatcher<Self>,
    ) -> (
        Self,
        impl irisia::structure::StructureCreate<
            Target = irisia::el_model::SharedEM<impl ElementInterfaces>,
        >,
    )
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

        let children = single::<Flex, _>(
            (),
            (),
            repeat(
                {
                    let vec = vec.clone();
                    wire(move || {
                        vec.read()
                            .iter()
                            .enumerate()
                            .map(|(index, x)| (index, x.clone()))
                            .collect()
                    })
                },
                |index, _| index,
                {
                    let vec = vec.clone();
                    move |(index, color)| {
                        let index = *index;
                        branch(
                            color.clone(),
                            {
                                let vec = vec.clone();
                                move |color| {
                                    single::<Rectangle, _>(
                                        {
                                            let color = color.clone();
                                            RectProps { force_color: color }
                                        },
                                        (),
                                        (),
                                        {
                                            let vec = vec.clone();
                                            move |access| {
                                                let vec = vec.clone();
                                                access.listen().spawn(move |_: PointerDown| {
                                                    vec.borrow_mut().remove(index);
                                                });
                                            }
                                        },
                                    )
                                }
                            },
                            (),
                        )
                    }
                },
            ),
            |_| {},
        );

        (App { rects: vec }, children)
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
