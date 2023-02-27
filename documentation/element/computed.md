## 计算值
计算值是一种概念，它和前端框架Vue.js的computed很像。计算值的目的有两点
- **提升性能**：将函数生成的值缓存起来，避免重复计算。
- **提升可读性**：在设置子元素属性时，`&prop: expr`语法将默认使用计算值。

计算值有分为两种，动态计算值和静态计算值，下面将详细讲解它们。

#### 动态计算值
动态计算值即生成结果含生命周期的值，它们不能被储存在元素中，它们只可由函数生成并储存在栈上。动态元素的设置很简单，直接作为普通方法声明即可。

```rust
use std::slice::Iter;

impl MyElement<'_>{
    fn calc(&self) -> Iter{
        self.vec().iter()
    }
}
```

#### 静态计算值
静态计算值可以缓存函数生成的结果，它要求函数生成的结果必须是`'static`的。需要用到计算值缓存器，简称计缓器。

```rust
struct MyElemCore{
    __computed1: Cache<String>,
    __data_raw_str: String,
}

struct MyElem<'a>{
    __core: &'a mut MyElemCore,
}

impl Deref for MyElem<'_>{
    ...
}

impl DerefMut for MyElem<'_>{
    ...
}

impl MyElem<'_>{
    fn greet(&self) -> &String {
        self.__computed1.read(|| {
            format!("hello, {}!", self.raw_str())
        })
    }

    fn raw_str(&self) -> &String {
        &self.__data_raw_str
    }

    fn raw_str_mut(&mut self) -> &mut String {
        self.__computed1.set_dirty();
        &mut self.__data_raw_str
    }
}
```

#### 内联计算值
在对子元素属性赋值时，可采用以下语法：
```rust
cream! {
    ...
    MyElem {
        name: self.name(),
        &greet: format!("hi, {}!", self.name())
    }
    ...
}
```
其中，`name`和`greet`都是`Elem`的prop。冒号后的值应该为一个表达式，它们将作为函数多次调用，且函数储存在父元素上。

但它们有一处不同，在于`greet`前有一个`&`。这意味着`greet`应当作为静态计算值处理而`name`则是动态计算值。但由于要储存在父元素上，且仅有一个表达式，我们不能用宏直接给出对应的类型。**因此，我们要求在子元素的模块内必须定义与所有props对应的`type`别名**。

在`my_elem`模块
```rust
mod my_elem {
    pub type TypeOfPropName = &'static str;
    pub type TypeOfPropGreet = String;

    pub struct MyElem {
        name: IntoBorrow<TypeOfPropName>,
        greet: IntoBorrow<TypeOfPropGreet>
    }

    ...
}
```

在父元素实现中
```rust
impl ... {
    fn anonymous_computed1(&self) -> my_elem::TypeOfPropName {
        self.name()
    }

    fn anonymous_computed2(&self) -> my_elem::TypeOfPropGreet {
        self.anonymous_computed2.read()
    }
}

```