use irisia::{
    application::Window,
    build,
    element::{Element, ElementUpdate},
    event::standard::Click,
    skia_safe::Color,
    style,
    style::StyleColor,
    ElModel,
};
use irisia_widgets::textbox::{
    styles::{StyleFontSize, StyleFontWeight},
    TextBox,
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
    type BlankProps = ();
}

impl ElementUpdate<()> for App {
    fn el_create(_this: ElModel!(), props: ()) -> Self {
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

    fn el_update(&mut self, this: ElModel!(), props: (), equality_matters: bool) -> bool {
        equality_matters
    }

    fn set_children(&self, this: ElModel!()) {
        this.set_children(build! {
            Flex {
                TextBox {
                    text: "Hello\nпpивeт\nこんにちは\n你好\n\nIrisia GUI🌺",
                    user_select: true,
                    +style: style!{
                        if 1 + 1 == 2 {
                            color: Color::MAGENTA;
                        }
                        font_weight: .bold;
                        font_size: 30px;
                    }
                }

                for (index, color) in self.rects.iter().enumerate() {
                    @key index;
                    Rectangle {
                        +style: style!{
                            width: 100px;
                            height: 100px + 40px * index as f32;
                            color: color.clone();
                        },
                        +oncreate: move |eh:&_| {
                            rect_rt(eh, index);
                        },
                    }
                }
            }
        })
        .layout_once(this.draw_region())
        .unwrap();
    }
}

fn rect_rt(this: ElModel!(Rectangle), index: usize) {
    println!("rectangle {index} got");
    this.listen().sys_only().spawn(move |_: Click, _| {
        println!("rectangle {} clicked", index);
    });
}
