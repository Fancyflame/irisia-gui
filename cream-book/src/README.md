# 简介

Cream GUI是一个基于Rust程序设计语言的一个跨平台，高性能的图形用户界面框架。它以
[winit](https://crates.io/crates/winit)
为窗口启动器，[skia](https://skia.org/)为渲染后端，[tokio](https://crates.io/crates/tokio)为异步执行器。

## 快速浏览

一个简单的窗体程序看起来是这样的。最新的例子可以移步github上的
[examples](https://github.com/Fancyflame/cream-rs/tree/main/examples)文件夹。

```rust,ignore
#[cream::main]
async fn main() {
    cream::new::<App>("test".into()).await.unwrap().join().await;
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

![渲染结果](window.jpg)
