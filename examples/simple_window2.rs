use irisia::{
    component::{hooks::UseHook, Comp, Component},
    model::{VModel, VNode},
    prim_element::{block::vmodel::Block, rect::Rect, text::Text},
    primitive::{Point, Region},
    skia_safe::Color,
    Result, Window, WinitWindow,
};

#[irisia::main]
async fn main() -> Result<()> {
    Window::new(
        WinitWindow::default_attributes().with_title("hello irisia"),
        || Comp {
            props: App.into(),
            child_data: (),
        },
    )
    .await
    .unwrap()
    .join()
    .await;
}

pub struct App;

impl Component for App {
    fn run(&self, use_hook: UseHook) -> impl VNode {
        println!("render");
        Comp {
            child_data: (),
            props: Average {
                children: (
                    Vec::from_iter(
                        std::iter::repeat_n(
                            Rect {
                                color: Color::GRAY,
                                border_radius: [50.0; 4],
                            },
                            3,
                        )
                        .enumerate(),
                    ),
                    Text {
                        text: "中文😀\n🎁".into(),
                        font_size: 60.0,
                        font_color: Color::BLACK,
                    },
                ),
            }
            .into(),
        }
    }
}

pub struct Average<T: VModel + Clone> {
    children: T,
}

impl<T: VModel + Clone> Component for Average<T> {
    fn run(&self, use_hook: UseHook) -> impl VNode {
        Block {
            children: self.children.clone(),
            layout_fn: |size, el, regions| {
                let width = size.0 / el.len() as f32;
                let height = size.1;

                for i in 0..el.len() {
                    let i = i as f32;
                    regions.push(Region::new(
                        Point(i * width, 0.0),
                        Point((i + 1.0) * width, height),
                    ));
                }
            },
        }
    }
}
