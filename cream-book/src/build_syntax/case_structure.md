# 选择结构

选择结构允许您根据条件选择表达式。它们会拓展为一个`enum`结构体，以储存不同类型的表达式。if和match表达式和rust的语法非常相似，除了表达式是元素声明语句。

## if表达式

```rust
build! {
    if 1 + 1 == 2 {
        Element1 {
            props: "1 + 1 == 2 is true"
        }
    }

    if 1 + 2 == 3 {
        Element1 {
            props: "1 + 2 == 3 is true"
        }
    } else {
        Element2 {
            other_props: "oh no, something wrong happened!"
        }
    }
}
```

## match表达式

```rust
build! {
    match 20 {
        1 => Element1 {
            props: "value is 1"
        },

        num if num % 2 == 0 => Element2 {
            other_props: "value can be exact divided by 2"
        }

        _ => {}
    }
}
```
