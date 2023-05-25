![irisia banner](images/banner_with_shadow_mirrored.jpg)

# Irisia GUI

Irisia GUI is a Rust-based GUI framework, featured high-performance, cross-platform, flexible, empowers everyone to build modern applications with accuracy and efficiency.

Irisia is heavily depend on following crates:

- [winit](https://crates.io/crates/winit): A window launcher, be widely used in Rust.
- [skia-safe](https://crates.io/crates/skia-safe): Bindings to [Skia](https://skia.org/). Skia is a graphics library developed by Google, used by lots of project, most famously Chrome and Android.
- [tokio](https://crates.io/crates/tokio): A asynchronous library with runtime and useful utils, high-speed and reliable.

**Irisia GUI is under development now, not ready for production yet.**

## 📕 Irisia Book

Only Chinese is available now\
目前只有中文可用

<https://fancyflame.github.io/irisia-gui/>

## Progress
The progress is now at *cache model*. Designing this part is time consuming, because cache model is closely
related to the animation system and the way elements rendering.
--2023/5/25

- [x] feasibility test
- [x] `build` and `style` macro
- [x] backend support (`skia` and `winit`)
- [x] text rendering 
- [ ] **cache model (with animation cache system)**
- [ ] animation system
- [ ] more widgets (button, textbox, checkbox, etc.)
- [ ] android support
- [ ] release

## 💻 CONTRIBUTING
Welcome to Irisia GUI! This is a project in progress and we're actively seeking contributors to help us develop and improve it.

### Project Status
We're in the early stages of development and there's a lot of work to be done. We're looking for developers, designers, and testers who are interested in contributing to this project.

We welcome all types of contributions, from code and documentation to bug reports and feature requests.

### How to contribute
Contributing documentation is lack now, all we have is examples in [examples directory](https://github.com/Fancyflame/irisia-gui/tree/main/examples) and some implementation of common components in [`irisia` crate](https://github.com/Fancyflame/irisia-gui/tree/main/irisia/src). Please *feel free* to ask any questions about this project in issues or discussion tab.

**If you are tend to participate in our project, please contact me via <fancyflame@163.com>. Thanks!**

## 🌍 About English Documentation

- We are sorry but due to my(or our) limited English proficiency, documentation is not available now. We will add as soon as possible.
- How about take a look at [`window`](https://github.com/Fancyflame/irisia-rs/blob/main/examples/window.rs) example?
- If you tend to translate the documentation into English, please feel free to open an issue to let me(us) know. Thanks a lot.
- **If you in need of English documentation, also please open an issue to let us know!**

## 👀 Take a quick look

A simple window application demo(located at [examples/simple_window.rs](https://github.com/Fancyflame/irisia-rs/tree/main/examples/simple_window.rs)) is looks like following. Newest examples please take a look at
[examples](https://github.com/Fancyflame/irisia-rs/tree/main/examples) directory.

```rust
use irisia::{
    application::Window,
    build,
    element::{Element, ElementHandle, NeverInitalized, NoProps, RuntimeInit},
    event::standard::Click,
    skia_safe::Color,
    structure::StructureBuilder,
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
            impl irisia::structure::VisitIter<Self::ChildProps<'a>>,
        >,
    ) -> irisia::Result<()> {
        build! {
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

fn rect_rt(eh: &ElementHandle, index: usize) {
    println!("rectangle {index} got");
    let eh = eh.clone();
    eh.clone().spawn(async move {
        loop {
            eh.recv_sys::<Click>().await;
            println!("rectangle {} clicked", index);
        }
    });
}

```

![rendering result](images/window.jpg)

## 💬 Discussion Guidelines

Welcome to the discussion section of this project. We encourage constructive and respectful communication among all participants.

### What to discuss
Please feel free to discuss any topics related to the project, such as:

- Ask for help about usage
- Bug reports and feature requests
- Code review and technical questions
- Suggestions and improvements
- Other relevant topics

### What not to discuss
To ensure a positive and inclusive environment, we ask that you refrain from discussing the following topics:

- Politics, religion, or other sensitive topics that are not relevant to the project
- Offensive or inappropriate language, behavior, or content
- Personal attacks, insults, or harassment of any kind

### Guidelines for communication
We encourage participants to follow these guidelines when communicating in the discussion section:

- Discuss in English if possible
- Be respectful and considerate of others' opinions and ideas
- Stay on a constructive tone and avoid negative or aggressive language
- Provide constructive feedback and suggestions for improvement when appropriate
- If you have a disagreement with someone, try to address it politely and professionally

Thank you for your cooperation in making this a positive and productive discussion environment.
