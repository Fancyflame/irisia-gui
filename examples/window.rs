//#![windows_subsystem = "windows"]
use irisia::{
    application::Window,
    box_styles::*,
    element::{Element, NeverInitalized, NoProps, RuntimeInit},
    event::standard::{Blured, Click, ElementCreated, Focused},
    render_fn,
    skia_safe::Color,
    style,
    style::StyleColor,
    textbox::{styles::*, TextBox},
};
use window_backend::{Flex, MyRequestClose, Rectangle, StyleHeight, StyleWidth};

mod window_backend;

#[irisia::main]
async fn main() {
    let win = Window::new::<App>("hello irisia").await;
    println!("window recv");
    win.unwrap().join().await;
}

struct App {
    rects: Vec<Color>,
}

impl Element for App {
    type Props<'a> = NoProps;
    type ChildProps<'a> = NeverInitalized;

    render_fn! {
        @init(self);
        Flex {
            TextBox {
                text: "Hello\n привет\n こんにちは\n 你好\n\n Irisia GUI🌺",
                user_select: true,
                +id: "textbox",
                +style: style!{
                    if 1 + 1 == 2{
                        color: Color::MAGENTA;
                    }
                    border: 5px, Color::GREEN, .round_cap;
                    border_radius: 30px;
                    box_shadow: 20px, Color::MAGENTA;
                    margin: .left 20px, .top 30px;
                    font_weight: .bold;
                    font_size: 30px;
                }
            }

            for (index, color) in self.rects.iter().enumerate() {
                @key index;
                Rectangle {
                    +id: ("rect", index),
                    +style: style!{
                        width: 100.0;
                        height: 100.0 + 40.0 * index as f32;
                        color: color.clone();
                    }
                }
            }
        }
    }

    // TODO: simplify callback mechanism
    fn create(init: RuntimeInit<Self>) -> Self {
        init.element_handle.clone().spawn(async move {
            let a = async {
                let mut lock = init.element_handle.lock();
                loop {
                    let ElementCreated { result, key } = lock
                        .get_element_checked(|(s, _): &(&'static str, usize)| *s == "rect")
                        .await;
                    println!("rectangle {} got!", key.1);

                    let init_cloned = init.clone();
                    init.element_handle.spawn(async move {
                        let a = init_cloned.on(
                            || result.recv_sys::<Focused>(),
                            |_, _| async { println!("rectangle {} gained focus", key.1) },
                        );

                        let b = init_cloned.on(
                            || result.recv_sys::<Blured>(),
                            |_, _| async { println!("rectangle {} lost focus", key.1) },
                        );

                        let c = init_cloned.on(
                            || result.recv_sys::<Click>(),
                            |_, _| async { println!("rectangle {} clicked", key.1) },
                        );

                        let d = init_cloned.on(
                            || result.recv::<MyRequestClose>(),
                            |_, _| async {
                                println!("close request event received(sent by {:?})", key.1);
                                init.close_handle.close();
                            },
                        );

                        tokio::join!(a, b, c, d);
                    });
                }
            };

            let mut lock2 = init.element_handle.lock();
            let b = async {
                let ele = lock2.get_element_by_id(&"textbox").await;
                loop {
                    ele.hover().await;
                    println!("cursor hovering on textbox");
                    ele.hover_canceled().await;
                    println!("cursor hovering canceled");
                }
            };

            tokio::join!(a, b);
        });

        Self {
            rects: vec![
                Color::RED,
                Color::YELLOW,
                Color::BLUE,
                Color::GREEN,
                Color::BLACK,
            ],
        }
    }
}
