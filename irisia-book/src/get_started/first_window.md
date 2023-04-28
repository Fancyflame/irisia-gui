# 第一个窗口

## 创建一个自定义元素

打开`main.rs`，修改为下面内容

```rust
use irisia::Element;

struct MyApp;
impl Element for MyApp {
    // ...
}
```

现在，我们将要为`MyApp`实现`Element` trait。\
首先，我们使用`irisia::render_fn`宏来为我们自动实现`render`函数。

```rust
use irisia::{Element, render_fn};

#[derive(Default)]
struct MyApp;

impl Element for MyApp {
    render_fn! {
        @init(self);
    }
}
```

`render_fn`宏的语法和`build`语法几乎完全相似，唯一区别就是`@init`函数。可以在这里找到[`build`宏](../build_syntax/index.html)的详细用法。由于rust十分注重宏的[卫生性](https://veykril.github.io/tlborm/decl-macros/minutiae/hygiene.html)，所以用户必须手动提供`self`关键字*使self可访问*。当然，您也可以使用任意有效标识符，例如`@init(foo)`，它们会编译为`fn render(foo: &mut Self, ..) ..`。
然后，我们添加一个文本框。

```rust
use irisia::{
    textbox::TextBox,
    Element,
    render_fn,
    element::{NeverInitalized, NoProps},
};

#[derive(Default)]
struct MyApp;

impl Element for MyApp {
    render_fn! {
        @init(self);
        TextBox {
            text: "hello world"
        }
    }

    fn create(_: irisia::element::RuntimeInit<Self>) -> Self {
        Self{}
    }

    type Props<'a> = NoProps;
    type ChildProps<'a> = NeverInitalized;
}
```

这样，我们的第一个元素就设计完成了。

## 添加main函数

我们现在需要创建一个窗口让它运行起来。我们添加一个程序入口，
让它启动一个窗口来将我们的元素作为根元素渲染。

```rust
use irisia::{
    textbox::TextBox,
    Element,
    Window,
    render_fn,
};

# #[derive(Default)]
# struct MyApp;
#
# impl Element for MyApp {
#     render_fn! {
#         @init(self);
#         TextBox {
#             text: "hello world"
#         }
#     }
# }
#
#[irisia::main]
async fn main() {
    Window::new::<MyApp>("my first app").await.unwrap().join().await;
}
```

点击运行即可看到一个标题为`my first app`的窗口中渲染的`hello world`字样了。

您可能会疑惑，`MyApp`是一个零长度的结构体，那么有缓存吗？是的，这也是有缓存的。`render_fn!`会帮我们接收一个`&mut CacheBox`参数，我们定义的`TextBox`，以及后续您添加的其他元素都会一并缓存在这个缓存盒里，由一个名为`AddChildCache`的结构体代理。为了让用户专注于应用设计，我们没有让用户接管这部分。但是请注意，如果您选择手动实现`render`方法，**请不要用CacheBox缓存不同类型的元素**，这样会导致debug模式下程序panic，release模式下大量性能开销和元素创建。

总而言之，如果您使用`render_fn`宏，请放心，irisia能够保证妥善保管元素的缓存。
