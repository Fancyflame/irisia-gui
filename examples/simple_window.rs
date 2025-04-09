use std::time::Duration;

use irisia::{
    build2,
    hook::Signal,
    model::prim_element::{Block, Rect},
    prim_element::{block::LayoutFn, rect::RectStyle, Element},
    primitive::{Point, Region},
    skia_safe::Color,
    Result, Window, WinitWindow,
};

#[irisia::main]
async fn main() -> Result<()> {
    Window::new(
        WinitWindow::default_attributes().with_title("hello irisia"),
        move || {
            let style = Signal::state(RectStyle {
                color: Color::RED,
                border_radius: [50.0; 4],
            });
            tokio::task::spawn_local({
                let style = style.clone();
                let mut interval = tokio::time::interval(Duration::from_secs(1));
                let mut red = false;
                async move {
                    loop {
                        interval.tick().await;
                        let mut w = style.write();
                        w.color = if red { Color::RED } else { Color::BLUE };
                        red = !red;
                    }
                }
            });

            build2! {
                Block {
                    layout_fn: average_divide as LayoutFn,
                    Rect {
                        style:= style.to_signal(),
                    }
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
        },
    )
    .await
    .unwrap()
    .join()
    .await;
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
