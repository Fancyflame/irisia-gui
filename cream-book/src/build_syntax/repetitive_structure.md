# 循环结构

循环结构可以让您生成多个相同类型的元素。但为了得到高效的缓存，循环结构中必须提供*键*。

键是用来确定每一个数据对应的元素缓存的。如果不使用键，那么一个迭代器中如果元素发生中间插入、中间删除、调换顺序，则会导致一连串元素的属性需要重新设置，进而可能导致大量元素需要改变其缓存，造成性能损耗。

下面展示了如果没有键，插入一个新数据的过程。

```text
elements   1  2  3   5  6  7
           |  |  |   |  |  |
data       1  2  3   5  6  7
                   ^
                   |
                 插入4
```

所有数据对应的缓存将会后移一位，并在末尾创建一个新元素。

```text
elements*  1  2  3  5  6  7 new
           |  |  |  |  |  |  |
data       1  2  3  4  5  6  7
```

再更新缓存。显而易见，这样的操作改变了这4个元素的缓存。

```text
                   |------------|
elements   1  2  3 | 4  5  6  7 |
           |  |  | | |  |  |  | |
data       1  2  3 | 4  5  6  7 |
                   |------------|
```

如果使用键，则第二步将变为：

```text
             在此插入新元素
                   |
                   v
elements*  1  2  3  5  6  7
           |  |  |    \  \  \
data       1  2  3  4  5  6  7
                    ^
                    |
                  插入4
```

```text
                  (new)
elements   1  2  3  4  5  6  7
           |  |  |  |  |  |  |
data       1  2  3  4  5  6  7
```

这样，一次只需改变（初始化）一个元素的缓存就行了。

## for表达式

for表达式中，`@key`指令是可选的。如果不指定`@key`,则默认使用迭代器元素作为键。这要求迭代器元素实现`Clone + Hash + Eq + 'static`，若不满足，则引起编译期错误。此时，需要您通过`@key`指令手动指定键。

```rust
build! {
    for num in 0..10 {
        Element1 {
            props: num
        }
    }
}
```

等同于

```rust
build! {
    for num in 0..10 {
        @key num;
        Element1 {
            props: num
        }
    }
}
```

下面这个例子将会引发编译期错误

```rust
let vec = vec![1, 2, 3];
build! {
    // 错误：num不满足`'static`
    for num in vec.iter() {
        Element1 {
            props: *num
        }
    }
}
```

解决办法是指定`@key`，或改为`iter().copied()`。

```rust
let vec = vec![1, 2, 3];
build! {
    for num in vec.iter() {
        @key *num;
        Element1 {
            props: *num
        }
    }
}
```

```rust
let vec = vec![1, 2, 3];
build! {
    for num in vec.iter().copied() {
        Element1 {
            props: *num
        }
    }
}
```

## while表达式

与for表达式不同的是，while表达式**必须指定**键。

```rust
let mut iter = vec![1, 2, 3].into_iter();
build! {
    while let Some(num) = iter.next() {
        @key num;
        Element1 {
            props: num
        }
    }
}
```

如果您不知道指定什么键，请考虑下面的方案。这将回归到本章开始部分的“无键缓存方案”。

```rust
build! {
    for _ in (0..).take_while(|_| expression_here) {
        Element1 {
            props: 0
        }
    }
}
```
