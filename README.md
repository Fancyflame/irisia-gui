![banner](images/banner_with_shadow_mirrored.jpg)

# Irisia GUI

Irisia GUI is a GUI framework based on Rust programming language, featured high-performance, cross-platform, flexible, empowers everyone to build an morden applications with accuracy and efficiency.

Irisia is heavily depend on following crates:

- [winit](https://crates.io/crates/winit): A window launcher, which is widely used in Rust.
- [skia-safe](https://crates.io/crates/skia-safe): Bindings to [Skia](https://skia.org/). Skia is a graphics library developed by Google, used by lots of project, most famously Chrome and Android.
- [tokio](https://crates.io/crates/tokio): A asynchronous library with runtime and useful utils, which is high-speed and reliable.

## 📕 Irisia Book

Only Chinese is available now\
目前只有中文可用

<https://fancyflame.github.io/irisia-gui/>

## 💻 CONTRIBUTING
Welcome to Irisia GUI! This is a project in progress and we're actively seeking contributors to help us develop and improve it.

### Project Status
We're in the early stages of development and there's a lot of work to be done. We're looking for developers, designers, and testers who are interested in contributing to this project.

We welcome all types of contributions, from code and documentation to bug reports and feature requests.

### How to contribute
Contributing documentation is lack now, all we have is examples in [examples directory](https://github.com/Fancyflame/irisia-gui/tree/main/examples) and some implementation of common components in [`irisia` crate](https://github.com/Fancyflame/irisia-gui/tree/main/irisia/src). Please *feel free* to ask any questions about this project in issues or discussion tab.

**If you are tend to participate in our project, please contact me via <fancyflame@163.com>. Thanks! ❤**

## 🌍 About English Documentation

- We are sorry but due to my(or our) limited English proficiency, documentation is not available now. We will add as soon as possible.
- How about take a look at [`window`](https://github.com/Fancyflame/irisia-rs/blob/main/examples/window.rs) example?
- If you tend to translate the documentation into English, please feel free to open an issue to let me(us) know. Thanks a lot.
- **If you in need of English documentation, also please open an issue to let us know!**

## 👀 Take a quick look

A simple window application is looks like following. Newest examples please take a look at
[examples](https://github.com/Fancyflame/irisia-rs/tree/main/examples) directory.

```rust
#[irisia::main]
async fn main() {
    irisia::new::<App>("test".into()).await.unwrap().join().await;
}

struct App {
    rects: Vec<Color>,
}

impl Default for App {
    fn default() -> Self {
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

impl Element for App {
    type Props<'a> = NoProps;
    type ChildProps<'a> = NeverInitalized;

    irisia::render_fn! {
        @init(self);
        Flex {
            TextBox {
                text: "hello世界🌏",
                +style: style!{
                    color: Color::MAGENTA;
                    font_slant: .normal;
                    font_size: 50px;
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

    fn start_runtime(init: RuntimeInit<Self>) {
        tokio::spawn(async move {
            loop {
                let event = init.event_dispatcher.recv_sys::<WindowEvent>().await;

                match event {
                    WindowEvent::MouseInput {
                        button: MouseButton::Left,
                        state: ElementState::Pressed,
                        ..
                    } => {
                        println!("left click");
                    }
                    _ => {}
                }
            }
        });
    }
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
