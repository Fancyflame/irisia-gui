use irisia::{
    application::Window,
    build,
    element::{Element, EventHandle, InitContent, NeverInitalized, NoProps},
    event::standard::Click,
    skia_safe::Color,
    structure_legacy::StructureBuilder,
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

    fn render<'a>(
        &mut self,
        mut frame: irisia::Frame<
            Self,
            impl style::StyleContainer,
            impl irisia::structure_legacy::VisitIter<Self::ChildProps<'a>>,
        >,
    ) -> irisia::Result<()> {
        build! {
            Flex {
                TextBox {
                    text: "Hello\nпpивeт\nこんにちは\n你好\n\nIrisia GUI🌺",
                    user_select: true,
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
                        +style: style!{
                            width: 100.0;
                            height: 100.0 + 40.0 * index as f32;
                            color: color.clone();
                        },
                        +oncreate: move |eh| {
                            rect_rt(eh, index);
                        },
                    }
                }
            }
        }
        .into_rendering(&mut frame.content)
        .finish(frame.drawing_region)
    }

    fn create(_: &InitContent<Self>) -> Self {
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

fn rect_rt(eh: &EventHandle, index: usize) {
    println!("rectangle {index} got");
    let eh = eh.clone();
    eh.clone().spawn(async move {
        loop {
            eh.recv_sys::<Click>().await;
            println!("rectangle {} clicked", index);
        }
    });
}
