## 样式
**样式是动态计算值**。它作为计算值的返回值是一个实现了`Style`的类型。

#### Traits

`Style` trait需要一个函数，尝试返回对应类型样式的值。

```rust
use std::any::{TypeId, Any};

trait Style {
    fn get_style_raw(&self, id: &TypeId) -> Option<&dyn Any>;
}
```

方便起见，还需另外提供拓展trait

```rust
trait StyleExt: Style {
    fn get_style<T>(&self) -> Option<&T>;
}
```

并提供实现

```rust
use std::any::TypeId;

impl<T> StyleExt for T
where
    T: Style
{
    fn get_style<T>(&self) -> Option<&T> {
        self
            .get_style_raw(&TypeId::of::<T>())
            .map(|x| x.downcast_ref().expect("type is incorrect"))
    }
}
```

#### 宏生成
首先，要自动生成一个样式结构体

```rust
use std::any::{TypeId, Any};

struct __AnonymousStyle1<'a>(&'a MyElem);

impl Style for __AnonymousStyle1<'_> {
    fn get_style_raw(&self, id: &TypeId) -> Option<&dyn Any> {
        if id == TypeId::of::<Type1>() {
            Some(self.style1())
        } else if id == TypeId::of::<Type2>() {
            Some(self.style2())
        } else {
            None
        }
    }
}

```