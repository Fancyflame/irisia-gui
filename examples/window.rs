use irisia::{
    application::Window,
    element::{Element, NeverInitalized, NoProps, RuntimeInit},
    event::standard::{Blured, Click, ElementCreated, Focused, PointerMove},
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

    fn create(init: RuntimeInit<Self>) -> Self {
        tokio::spawn(async move {
            let a = async {
                loop {
                    let ElementCreated { result, key } = init
                        .element_handle
                        .get_element_checked(|(s, _): &(&str, usize)| *s == "rect")
                        .await;

                    tokio::spawn(async move {
                        println!("rectangle {} got!", key.1);

                        loop {
                            tokio::select! {
                                _ = result.recv_sys::<Focused>() => {
                                    println!("rectangle {} gained focus",key.1);
                                }
                                _ = result.recv_sys::<Blured>() => {
                                    println!("rectangle {} lost focus",key.1);
                                }
                                _ = result.recv_sys::<Click>() => {
                                    println!("rectangle {} clicked", key.1);
                                }
                                _ = result.recv::<MyRequestClose>() => {
                                    println!("close request event received(sent by {:?})", key.1);
                                    init.close_handle.close();
                                    break;
                                }
                            }
                        }
                    });
                }
            };

            let b = async {
                let ele = init.element_handle.get_element_eq(&"textbox").await;
                ele.recv_sys::<PointerMove>().await;
                tokio::spawn(async move {
                    loop {
                        ele.hover().await;
                        println!("cursor hovering on textbox");
                        ele.hover_canceled().await;
                        println!("cursor hovering canceled");
                    }
                });
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
