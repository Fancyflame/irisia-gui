//! ## 样式格式
//! 对于下列代码
//! ```no_run
//! irisia::style!{
//!     box_shadow:
//!         10px, 10px,
//!         .outset,
//!         .blur 20px,
//!         .color Color::SKYBLUE,
//!         .pass_tuple ("bar", 7px)
//!     ;
//! }
//! ```
//! 1. **`box_shadow`**:
//!   将会转换成类型储存在样式集结构体中，并在初始化时执行`StyleBoxShadow::style_create((Pixel(10), Pixel(10)))`
//!
//! 2. **`10px`**:
//!   `px`, `pct`等后缀属于框架内置的固定数字表示法。`10px`将转换成`irisia::Pixel(10)`
//!
//! 3. **`.outset`**:
//!   以`.`开头且无参数，`style.outset()`
//!
//! 4. **`.blur 20px`**:
//!   以`.`开头有参数，`style.blur(Pixel(20))`，至多只允许一个参数
//!

pub mod anima;

#[derive(Clone, Copy)]
pub enum StyleValue {
    Delimiter,
    Float(f32),
    Ident(&'static str),
    Color([u8; 4]),
}

pub trait StyleSource {
    fn get_style<'a>(&'a self, name: &str, prog: f32) -> Option<&'a [StyleValue]>;
}

pub trait TupleStyleGroup: StyleSource {
    type Chained<T: StyleSource>: TupleStyleGroup;

    fn chain<T>(self, other: T) -> Self::Chained<T>
    where
        Self: Sized,
        T: StyleSource;
}

type Pair<T> = (&'static str, T);

impl<T> StyleSource for Pair<T>
where
    T: AsRef<[StyleValue]>,
{
    fn get_style<'a>(&'a self, name: &str, _: f32) -> Option<&'a [StyleValue]> {
        if name == self.0 {
            Some(self.1)
        } else {
            None
        }
    }
}

macro_rules! impl_tsg {
    () => {};
    (# $($T:ident)*)=>{
        impl<$($T,)*> TupleStyleGroup for ($($T,)*)
        where
            $($T: StyleSource,)*
        {
            type Chained<T_: StyleSource> = ((T_,), $($T,)*);

            fn chain<T>(self, other: T) -> Self::Chained<T>
            where
                Self: Sized,
                T: StyleSource
            {
                #[allow(non_snake_case)]
                let ($($T,)*) = self;
                (other, $($T,)*)
            }
        }

        impl<$($T,)*> StyleSource for ($($T,)*)
        where
            $($T: StyleSource,)*
        {
            fn get_style<'a>(&'a self, name: &str, prog: f32) -> Option<&'a [StyleValue]> {
                #[allow(non_snake_case)]
                let ($($T,)*) = self;

                $(if let arr @ Some(_) = $T.get_style(name, prog) {
                    arr
                } else)* {
                    None
                }
            }
        }

        impl<First, $($T,)*> TupleStyleGroup for (First, $($T,)*)
        where
            $($T: StyleSource,)*
            First: TupleStyleGroup,
        {
            type Chained<T_: StyleSource> = (First::Chained<T_>, $($T,)*);

            fn chain<T>(self, other: T) -> Self::Chained<T>
            where
                Self: Sized,
                T: StyleSource,
            {
                #[allow(non_snake_case)]
                let (first, $($T,)*) = self;
                (first.chain(other), $($T,)*)
            }
        }

        impl<First, $($T,)*> StyleSource for (First, $($T,)*)
        where
            $($T: StyleSource,)*
            First: StyleSource,
        {
            fn get_style<'a>(&'a self, name: &str, prog: f32) -> Option<&'a [StyleValue]> {
                #[allow(non_snake_case)]
                let (first, $($T,)*) = self;

                if let arr @ Some(_) = first.get_style(name, prog) {
                    arr
                }$(else if let arr @ Some(_) = $T.get_style(name, prog) {
                    arr
                })* else {
                    None
                }
            }
        }

        impl_tsg!($($T)*);
    };
    ($T1:ident $($T:ident)*) => {
        impl<$($T,)*> TupleStyleGroup for ($($T,)*)
        where
            $($T: StyleSource,)*
        {
            type Chained<T_: StyleSource> = (T_, $($T,)*);

            fn chain<T>(self, other: T) -> Self::Chained<T>
            where
                Self: Sized,
                T: StyleSource
            {
                #[allow(non_snake_case)]
                let ($($T,)*) = self;
                (other, $($T,)*)
            }
        }

        impl<$($T,)*> StyleSource for ($($T,)*)
        where
            $($T: StyleSource,)*
        {
            fn get_style<'a>(&'a self, name: &str, prog: f32) -> Option<&'a [StyleValue]> {
                #[allow(non_snake_case)]
                let ($($T,)*) = self;

                $(if let arr @ Some(_) = $T.get_style(name, prog) {
                    arr
                } else)* {
                    None
                }
            }
        }

        impl_tsg!($($T)*);
    };
}

impl_tsg!(
    # A B C D E F G H I J K L M N O
    A1 B1 C1 D1 E1 F1 G1 H1 I1 J1 K1 L1 M1 N1 O1
);
