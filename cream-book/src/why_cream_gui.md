# 为什么选择cream？
您或许有疑问。Rust中有那么多GUI框架可供挑选，cream有什么出众之处呢？

### 跨平台
得益于`winit`和`skia-safe`良好的跨平台性，cream可以跨桌面端和移动端。对Windows、Linux和Android平台的支持是我们的工作重心。

### 缓存
如果您阅读了[前一章节](index.html)，您或许会注意到一个`for`循环结构。请不用担心，我们将会对任何结构——顺序、选择和循环结构中所有元素进行缓存，使每一帧都以最小开销渲染。

或许您之前使用过React框架，与之不同的是cream-gui的循环语句可以储存任意可以被[HashMap](https://doc.rust-lang.org/std/collections/struct.HashMap.html#)作为键的结构体，但是强制您提供一个键。尽管这在`for`语句中，`@key`指令并不是必须的，那是因为在未指定`@key`时，默认使用迭代器中的元素作为键。

### 异步系统
cream内部分为两种工作模式：非阻塞和异步。

如果您不是一个组件开发者，您将很小概率和必须非阻塞代码部分打交道。必须非阻塞的代码大多作用在渲染部分，它要求不能等待，必须高效渲染每一帧。

相比之下，异步代码几乎全部作用在事件处理部分，您可以将等待事件和其他如文件系统和网络等异步IO更好地协作，这样能很好地提升工作效率，并避免回调地狱（Callback hell）。

### 高度自定义
cream允许您最大限度地自定义您的元素，您可以自定义元素布局、组件和样式。用户是可以还原出标准库中的文本框`TextBox`的。

在具有高自定义度的同时，我们也同样重视开发效率。如果您只想利用现成的模块快速构建您的应用程序，请多多熟悉`build`和`style`（以及`render_fn`）宏吧！

### 宏
cream很大程度上利用了rust的元编程系统。如前面您所见的整个库的核心宏`build`和`style`。除此之外，也有专门用于自定义样式的派生宏`Style`等。
```rust
use cream::{Style, Pixel, skia_safe::Color};

#[derive(Style)]
#[cream(
    from = "x_offset, y_offset[, blur_radius][, spread_radius][, color]",
    impl_default
)]
struct StyleBoxShadow {
    #[cream(default)]
    x_offset: Pixel,

    #[cream(default)]
    y_offset: Pixel,

    #[cream(default)]
    blur_radius: Pixel,

    #[cream(default)]
    spread_radius: Pixel,

    #[cream(default = "Color::BLACK")]
    color: Color
}

fn main() {
    let style = style! {
        box_shadow: 0px, 0px;
        box_shadow: 0px, 0px, 10px;
        // ...
    };
}
```