use std::time::Duration;

use irisia::{
    build2,
    hook::Signal,
    model::{
        prim_element::{Block, Rect},
        VModel,
    },
    prim_element::{block::LayoutFn, rect::RectStyle, Element},
    primitive::{Point, Region},
    skia_safe::Color,
    Result, Window, WinitWindow,
};

#[irisia::main]
async fn main() -> Result<()> {
    Window::new(
        WinitWindow::default_attributes().with_title("hello irisia"),
        app,
    )
    .await
    .unwrap()
    .join()
    .await;
}

fn app() -> impl VModel {
    let count_write = Signal::state(0);

    Signal::memo(
        |count| {
            build2! {
                Text {
                    text: format!("you clicked {count} times"),
                    on_click: move |_| {
                        *count_write.write() += 1;
                    },
                }
                MyComp {
                    Text {
                        text: format!("count count {count}"),
                    }
                }
            }
        },
        count_write.to_signal(),
    )
}
