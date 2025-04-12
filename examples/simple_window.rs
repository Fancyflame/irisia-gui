use std::time::Duration;

use irisia::{
    build2,
    hook::Signal,
    model::{
        prim_element::{Block, Rect, Text},
        VModel,
    },
    prim_element::{block::LayoutFn, rect::RectStyle, text::TextStyle, Element},
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
    let red_rect = Signal::state(false);
    let changing_rect = Signal::memo_ncmp(
        |&is_red| {
            build2! {
                Rect {
                    style: RectStyle {
                        color: if is_red { Color::RED } else { Color::BLUE },
                        border_radius: [50.0; 4],
                    },
                }
            }
        },
        red_rect.to_signal(),
    );
    let text = Signal::memo_ncmp(
        |&is_red| format!("当前显示{}色", if is_red { "红" } else { "蓝" }),
        red_rect.to_signal(),
    );

    tokio::task::spawn_local({
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        async move {
            loop {
                interval.tick().await;
                let mut w = red_rect.write();
                *w = !*w;
            }
        }
    });

    build2! {
        Block {
            layout_fn: average_divide as LayoutFn,

            Text {
                text:= text,
                style: TextStyle {
                    font_size: 40.0,
                    font_color: Color::MAGENTA,
                },
            }

            (changing_rect)

            for i in 0..2, key = i {
                Rect {
                    style: RectStyle {
                        color: Color::BLACK,
                        border_radius: [50.0; 4],
                    },
                }
            }
        }
    }
}

fn average_divide(size: Point, el: &[Element], regions: &mut Vec<Region>) {
    let width = size.0 / el.len() as f32;
    let height = size.1;

    for i in 0..el.len() {
        let i = i as f32;
        regions.push(Region::new(
            Point(i * width, 0.0),
            Point((i + 1.0) * width, height),
        ));
    }
}
