//#![windows_subsystem = "windows"]
use irisia::{
    application::{CloseHandle, Window},
    box_styles::*,
    build,
    element::{Element, ElementHandle, NeverInitalized, NoProps, RuntimeInit},
    event::standard::{Blured, Click, Focused},
    skia_safe::Color,
    structure::StructureBuilder,
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

    fn render<'a>(
        &mut self,
        mut frame: irisia::Frame<
            Self,
            impl style::StyleContainer,
            impl irisia::structure::VisitIter<Self::ChildProps<'a>>,
        >,
    ) -> irisia::Result<()> {
        build! {
            Flex {
                TextBox {
                    text: "Hello\npивeт\nこんにちは\n你好\n\nIrisia GUI🌺",
                    user_select: true,
                    +style: style! {
                        if 1 + 1 == 2 {
                            color: Color::MAGENTA;
                            ~: Color::MAGENTA;
                        }
                        border: 5px, Color::GREEN, .round_cap;
                        border_radius: 30px;
                        box_shadow: 20px, Color::MAGENTA;
                        margin: .left 20px, .top 30px;
                        font_weight: .bold;
                        font_size: 30px;
                    },
                    +oncreate: move |eh| {
                        eh.spawn(textbox_rt(eh.clone()));
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
                            let eh_cloned = eh.clone();
                            eh.spawn(rect_rt(eh_cloned, frame.ri.close_handle, index));
                        }
                    }
                }
            }
        }
        .into_rendering(&mut frame.content)
        .finish(frame.drawing_region)
    }

    fn create(_: &RuntimeInit<Self>) -> Self {
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

async fn rect_rt(eh: ElementHandle, close_handle: CloseHandle, key: usize) {
    println!("rectangle {} got!", key);

    eh.listen()
        .recv_sys()
        .spawn(move |_: Focused| println!("rectangle {} gained focus", key));

    eh.listen()
        .recv_sys()
        .spawn(move |_: Blured| println!("rectangle {} lost focus", key));

    eh.listen()
        .recv_sys()
        .spawn(move |_: Click| println!("rectangle {} clicked", key));

    eh.listen().spawn(move |_: MyRequestClose, _| {
        println!("close request event received(sent by {})", key);
        close_handle.close();
    });
}

async fn textbox_rt(eh: ElementHandle) {
    loop {
        eh.hover().await;
        println!("cursor hovering on textbox");
        eh.hover_canceled().await;
        println!("cursor hovering canceled");
    }
}
