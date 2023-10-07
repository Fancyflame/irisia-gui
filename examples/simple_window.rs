use irisia::{
    application::Window,
    build,
    element::{Element, ElementUpdate},
    event::standard::{CloseRequested, PointerDown},
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
    rects: Vec<Option<Color>>,
}

impl Element for App {
    type BlankProps = ();

    fn set_children(&self, this: &ElModel!()) {
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

impl ElementUpdate<()> for App {
    fn el_create(this: &ElModel!(), _: ()) -> Self {
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

    fn el_update(&mut self, _: &ElModel!(), _: (), _: bool) -> bool {
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
            this.el_write().await.unwrap().rects[index].take();
        });
}
