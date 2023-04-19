use irisia::{
    application::Window,
    element::{Element, NeverInitalized, NoProps, RuntimeInit},
    event::standard::{Click, ElementCreated},
    render_fn,
    skia_safe::Color,
    style,
    style::StyleColor,
    textbox::{styles::*, TextBox},
};
use window_backend::{Flex, Rectangle, StyleHeight, StyleWidth};

mod window_backend;

#[irisia::main]
async fn main() {
    Window::new::<App>("hello irisia")
        .await
        .unwrap()
        .join()
        .await;
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
        let handle = init.element_handle.clone();
        init.element_handle.spawn(async move {
            loop {
                let ElementCreated { result, key } = handle
                    .get_element_checked(|(s, _): &(&str, usize)| *s == "rect")
                    .await;

                tokio::spawn(async move {
                    loop {
                        result.recv_sys::<Click>().await;
                        println!("rectangle {} clicked", key.1);
                    }
                });
            }
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
