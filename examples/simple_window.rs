use std::{rc::Rc, time::Duration};

use irisia::{
    application::Window,
    data_flow::{
        register::{register, Register},
        wire, Readable, ReadableExt,
    },
    element::{Component, ComponentTemplate, ElementInterfaces},
    event::standard::PointerDown,
    skia_safe::Color,
    structure::{pat_match, repeat, single, RepeatMutator},
    winit::window::WindowBuilder,
    Result,
};

use window_backend::{Flex, RectProps, Rectangle, StyleHeight, StyleWidth};

mod window_backend;

#[irisia::main]
async fn main() -> Result<()> {
    Window::new(
        WindowBuilder::new().with_title("hello irisia"),
        single::<Component<App>>((), (), (), |_| {}),
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

        let children = single::<Flex>(
            (),
            (),
            repeat({
                let vec = vec.clone();
                move |mutator| {
                    mutator.update(
                        vec.read().iter().cloned().enumerate(),
                        |&(index, _)| index,
                        |item| {
                            pat_match(
                                {
                                    let item = item.clone();
                                    wire(move || *item.read().1.read())
                                },
                                {
                                    let vec = vec.clone();
                                    let item = item.clone();
                                    move |color| {
                                        single::<Rectangle>(
                                            RectProps {
                                                force_color: color.clone(),
                                            },
                                            (),
                                            (),
                                            {
                                                let vec = vec.clone();
                                                let item = item.clone();
                                                move |access| {
                                                    access.listen().spawn(move |_: PointerDown| {
                                                        let value = item.read().0;
                                                        vec.borrow_mut().remove(value);
                                                    });
                                                }
                                            },
                                        )
                                    }
                                },
                                (),
                            )
                        },
                    )
                }
            }),
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
