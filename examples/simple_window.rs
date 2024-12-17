use std::time::Duration;

use irisia::{
    application::Window,
    build,
    el_model::ElementAccess,
    element::{get_empty_props, Component, ComponentTemplate},
    event::standard::{PointerDown, PointerEntered, PointerOut},
    hook::{Provider, ProviderObject, Signal, ToProviderObject},
    model::{unit::Unit, RootDesiredModel},
    skia_safe::Color,
    ElementInterfaces, Result, WinitWindow,
};

use tokio::select;
use window_backend::{Flex, Rectangle};

mod window_backend;

#[irisia::main]
async fn main() -> Result<()> {
    Window::new(
        WinitWindow::default_attributes().with_title("hello irisia"),
        || Unit::<Component<App>, _, _, _> {
            props: (),
            child_data: (),
            slot: (),
            on_create: |_| {},
        },
    )
    .await
    .unwrap()
    .join()
    .await;
}

struct App;

impl ComponentTemplate for App {
    type Props = ();

    fn create<Slt>(
        props: &Self::Props,
        access: ElementAccess,
        slot: ProviderObject<Slt>,
    ) -> ProviderObject<impl irisia::model::DesiredVModel<()> + 'static>
    where
        Slt: irisia::model::DesiredVModel<()> + 'static,
    {
        let vec = Signal::state(
            [
                Color::RED,
                Color::YELLOW,
                Color::BLUE,
                Color::MAGENTA,
                Color::BLACK,
            ]
            .into_iter()
            .collect::<Vec<_>>(),
        );

        let children = Signal::builder(structure(&vec))
            .dep(
                {
                    let vec = vec.clone();
                    move |mut setter, _| {
                        *setter = structure(&vec);
                    }
                },
                vec.clone(),
            )
            .build()
            .to_object();

        tokio::task::spawn_local({
            async move {
                loop {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    if vec.read().len() < 6 {
                        let mut w = vec.write();
                        w.push(Color::RED);
                    }
                }
            }
        });

        children
    }
}

fn structure(rects: &Signal<Vec<Color>>) -> impl RootDesiredModel<()> {
    let repeat: Vec<_> = rects
        .read()
        .iter()
        .enumerate()
        .map(|(index, color)| {
            let height = Signal::state(Some(40.0));
            (
                index,
                Unit::<Rectangle, _, _, _> {
                    props: get_empty_props::<Rectangle>()
                        .color(Some(color.clone()))
                        .height(height.clone()),
                    child_data: (),
                    slot: (),
                    on_create: {
                        let rects = rects.clone();
                        move |access: &ElementAccess| {
                            let rects = rects.clone();
                            access.listen().spawn(move |_: PointerDown| {
                                rects.write().remove(index);
                            });
                        }
                    },
                },
            )
        })
        .collect();

    Unit::<Flex, _, _, _> {
        props: (),
        child_data: (),
        slot: repeat,
        on_create: |_| {},
    }
}

// fn structure(rects: &RegVec<(usize, Option<Color>)>) -> impl RootStructureCreate {
//     build! {
//         input rects;

//         Flex {
//             for (index, item) in rects.clone() {
//                 if let Some(color) = item.clone() {
//                     let height = Register::new(40.0);
//                     Rectangle {
//                         color: {
//                             let c = color.clone();
//                             std::mem::forget(c);
//                             wire!(Color::RED)
//                         },
//                         height: wire!(Some(*height.read()); height),
//                         @on_create: move |access| {
//                             bind_rt(access.clone(), height.clone());
//                             access.listen().spawn(move |_: PointerDown| {
//                                 rects.write().remove(*index.read());
//                                 reset_index(&rects);
//                             });
//                         },
//                     }
//                 }
//             }
//         }
//     }
// }

fn bind_rt(access: ElementAccess, height: Signal<f32>) {
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
