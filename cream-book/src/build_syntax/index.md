# build语法

`build`宏可帮助您快速构建一个节点树。现在，先来学习一下怎样声明一个元素。

声明一个元素分为以下几部分：
```
<元素名> {
    (
        <属性名>: <表达式>,
        +<元素指令>: <表达式>,
    )*

    <子节点列表>
}
```
这是一个带有`text`和`select`属性的`TextBox`元素
```rust
TextBox {
    text: "this is a &str",
    select: true
}
```
目前，只有两个指令，`style`和`listen`。其中，`style`指令可向该元素提供一个实现了`StyleContainer` trait的结构体，`listen`则将表达式的值作为key，向该元素注册所有事件监听。这两个指令相关章节将在后面给出。