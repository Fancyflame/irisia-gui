# 为什么选择irisia？

## 高性能和高表现力

irisia采用winit作为窗口启动器，是rust最热门的窗口启动器之一。

irisia采用skia作为渲染后端。skia是由谷歌公司开发的跨平台图形库，应用于Chrome浏览器和Android原生渲染。经过长时间的考验，其稳定性和性能都十分出类拔萃。除此之外，skia拥有非常完善的字体渲染模块和绘制功能，使得irisia能在您的屏幕上呈现更多精彩画面。

## 跨平台

得益于`winit`和`skia-safe`良好的跨平台性，irisia可以跨桌面端和移动端。对Windows、Linux和Android平台的支持是我们的工作重心。

## 缓存

如果您阅读了[前一章节](index.html)，您或许会注意到各种语法结构。请不用担心，我们将会对任何语法结构——顺序、选择和循环结构中所有元素进行缓存，使每一帧都以最小开销渲染。

或许您之前使用过React框架，与之不同的是irisia的循环语句可以储存任意可以被[HashMap](https://doc.rust-lang.org/std/collections/struct.HashMap.html#)作为键的结构体，但是强制您提供一个键。尽管这在`for`语句中，`@key`指令并不是必须的，那是因为在未指定`@key`时，默认使用迭代器中的元素作为键。

## 异步系统

irisia内部分为两种工作模式：同步代码和异步代码。

如果您不是一个组件开发者，您将很小概率和同步代码部分打交道。同步代码大多作用在渲染部分，它要求不能等待，必须高效渲染每一帧。

相比之下，异步代码几乎全部作用在事件处理部分，您可以将等待事件和其他如文件系统和网络等异步IO更好地协作，这样能很好地提升工作效率，并避免回调地狱（Callback hell）。

## 高度自定义

irisia允许您最大限度地自定义您的元素，您可以自定义元素布局、组件和样式。用户是可以还原出标准库中的文本框`TextBox`的。

在具有高自定义度的同时，我们也同样重视开发效率。如果您只想利用现成的模块快速构建您的应用程序，请多多熟悉`build`和`style`（以及`render_fn`）宏吧！

## 丰富的宏

irisia很大程度上利用了rust的元编程系统。如前面您所见的整个库的核心宏`build`和`style`。除此之外，也有专门用于自定义样式的派生宏`Style`等。

```rust
use irisia::{Style, Pixel, skia_safe::Color};

#[derive(Style)]
#[irisia(
    from = "x_offset, y_offset[, blur_radius][, spread_radius][, color]",
    impl_default
)]
struct StyleBoxShadow {
    #[irisia(default)]
    x_offset: Pixel,

    #[irisia(default)]
    y_offset: Pixel,

    #[irisia(default)]
    blur_radius: Pixel,

    #[irisia(default)]
    spread_radius: Pixel,

    #[irisia(default = "Color::BLACK")]
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
