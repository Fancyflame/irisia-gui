use std::time::Duration;

use irisia::{
    application::Window,
    build,
    data_flow::{const_wire, wire2, ReadWire, Readable, ReadableExt, RegVec, Register, ToReadWire},
    el_model::ElementAccess,
    element::{ComponentTemplate, RootStructureCreate},
    event::standard::{PointerDown, PointerEntered, PointerMove, PointerOut, PointerUp},
    skia_safe::Color,
    style, wire, Result, WinitWindow,
};

use rand::Rng;
use tokio::select;
use window_backend::{Flex, Rectangle};

mod window_backend;

#[irisia::main]
async fn main() -> Result<()> {
    Window::new(
        WinitWindow::default_attributes().with_title("hello irisia"),
        build!(App;),
    )
    .await
    .unwrap()
    .join()
    .await;
}

struct App;

impl ComponentTemplate for App {
    type Props<'a> = ();

    fn create<Slt>(_: Self::Props<'_>, _: Slt, _: ElementAccess) -> impl RootStructureCreate
    where
        Slt: irisia::structure::StructureCreate<()>,
    {
        let vec = Register::new(
            [
                Color::RED,
                Color::YELLOW,
                Color::BLUE,
                Color::MAGENTA,
                Color::BLACK,
            ]
            .into_iter()
            .enumerate()
            .map(|(i, x)| Register::new((i, Some(x))))
            .collect::<Vec<_>>(),
        );

        let children = structure(&vec);

        tokio::task::spawn_local({
            async move {
                loop {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    if vec.read().len() < 5 {
                        let mut w = vec.write();
                        let len = w.len();
                        w.push(Register::new((len, Some(Color::RED))));
                    }
                }
            }
        });

        children
    }
}

fn structure(rects: &RegVec<(usize, Option<Color>)>) -> impl RootStructureCreate {
    build! {
        input rects;

        Flex {
            for (index, item) in rects.clone() {
                if let Some(color) = item.clone() {
                    let height = Register::new(40.0);
                    Rectangle {
                        color: {
                            let c = color.clone();
                            std::mem::forget(c);
                            wire!(Color::RED)
                        },
                        height: wire!(Some(*height.read()); height),
                        @on_create: move |access| {
                            bind_rt(access.clone(), height.clone());
                            access.listen().spawn(move |_: PointerDown| {
                                rects.write().remove(*index.read());
                                reset_index(&rects);
                            });
                        },
                    }
                }
            }
        }
    }
}

fn reset_index(rects: &RegVec<(usize, Option<Color>)>) {
    for (i, reg) in rects.read().iter().enumerate() {
        reg.write().0 = i;
    }
}

fn bind_rt(access: ElementAccess, height: Register<f32>) {
    let access2 = access.clone();
    access2.listen().spawn(move |_: PointerEntered| {
        let access = access.clone();

        let height = height.clone();
        let repeater = async move {
            let mut interval = tokio::time::interval(Duration::from_millis(50));
            let mut step = 1.0;
            loop {
                interval.tick().await;
                let mut height_w = height.write();
                if *height_w >= 200.0 {
                    step = -1.0;
                } else if *height_w <= 20.0 {
                    step = 1.0;
                }
                *height_w += step * 4.0;
            }
        };

        let access = access.clone();
        async move {
            select! {
                _ = access.event_dispatcher().recv::<PointerOut>() => {
                    println!("pointer out");
                }
                _ = repeater => {}
            }
        }
    });
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
