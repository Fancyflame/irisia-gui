# 顺序、嵌套和扩展

学习了如何声明一个元素，这篇章节将十分利于理解。

## 顺序结构

直接向下排列即可，*不能加逗号*。

```rust
build! {
    Element1;
    Element2;

    Element3 {
        props: "value"
    }
}
```

## 嵌套结构

可以通过这种方式声明子元素。可添加独立的花括号来分割您的内容。

```rust
build! {
    Div {
        Div {
            Element1;
        }

        {
            Element2;
            Element3 {
                props: "value"
            }
        }

        {
            Element4;
            Element5
        }
    }
}
```

## 拓展指令

可以通过`@extend <表达式>`语法来将其他元素树扩展到当前元素树。扩展的树*允许*多元素。

```rust
let ext = build! {
    Element1;
    Element2;
};

let branch = build! {
    @extend ext;
    Element3 {
        props: "value"
    }
    Element4;
};
```

## 插槽指令

如果在`build`的`@init`指令中提供了插槽（Slot），或使用`render_fn`宏，则可以使用`@slot`指令来将插槽中的所有元素合并到当前树中。它和手动`@extend <插槽表达式>`的效果是一样的。

```rust
render_fn! {
    Element1;
    Element2;
    @slot;
}
```
