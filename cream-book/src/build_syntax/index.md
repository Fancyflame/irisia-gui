# build语法

`build`宏可帮助您快速构建一个节点树。

## `@init`指令

`@init`是提供给`build`或`render_fn`宏基本参数的指令。一个宏最多只能有一个`@init`指令，而且*必须*出现在开头位置。

- 在`build`宏中，`@init`指令是可选的，可接受0~2个参数。语法为`@init[([<EventDispatcher>[, <Slot>]])];`。例如：
  - `@init;`：静态节点。
  - `@init(event_dispatcher);`：将监听元素创建事件。
  - `@init(event_dispathcer, slot);`：将监听元素创建事件，并且可使用一次`@slot`指令。
- 在`render_fn`宏中，`@init`指令是必须的，只能接受1个参数。语法为`@init(<ident>)`。例如：
  - `@init(self);`：等同于`fn render(&mut self, ..)..`。
  - `@init(foo);`：等同于`fn render(foo: &mut Self, ..)..`。

## 声明元素

声明一个元素分为以下几部分：

```text
<元素名> {
    (
        <属性名>: <表达式>,
        +<元素指令>: <表达式>,
    )*

    <子节点列表>
}
```

这是一个带有`text`和`select`属性的`TextBox`元素。

```rust
TextBox {
    +id: ("text_box", 12345),
    text: "this is a &str",
    select: true,
}
```

*元素内*指令和属性的不同之处在于它是以`+`开头的。目前，只有两个指令，`style`和`id`。其中，`style`指令可向该元素提供一个实现了`StyleContainer` trait的结构体，`id`则将表达式的值作为该元素的标识符，监听该元素的元素创建事件。关于这两个指令将在后面章节给出。
下面，来学习一下顺序结构、嵌套结构和拓展指令。
