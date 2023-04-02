# 简介
Cream GUI是一个基于Rust程序设计语言的一个跨平台，高性能的图形用户界面框架。它以
[winit](https://crates.io/crates/winit)
为窗口生成器，[skia](https://skia.org/)为渲染后端，[tokio](https://crates.io/crates/tokio)为异步执行器。

# 快速浏览
一个简单的窗体程序看起来是这样的。最新的例子可以移步github上的
[examples](https://github.com/Fancyflame/cream-rs/tree/main/examples)文件夹。
```rust,ignore
#[cream::main]
async fn main() {
    cream::new::<App>("my window".into()).await.unwrap().recv_destroyed().await;
}

struct App {
    rects: Vec<Color>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            rects: vec![Color::GREEN, Color::RED, Color::BLUE],
        }
    }
}

impl Element for App {
    type Props<'a> = NoProps;
    type ChildProps<'a> = NeverInitalized;

    cream::render_fn! {
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
                    +listen: ("rect", index),
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
                let (event, _) = init
                    .event_dispatcher
                    .recv::<WindowEvent, ElementEvent>()
                    .await;

                if let WindowEvent::MouseInput {
                    button: MouseButton::Left,
                    state: ElementState::Pressed,
                    ..
                } = event {
                    println!("mouse pressing");
                }
            }
        });
    }
}
```
