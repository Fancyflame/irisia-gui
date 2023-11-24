use super::{Style, StyleGroup};

pub trait StyleReader {
    fn read_style(group: impl StyleGroup) -> Self;
}

impl<T> StyleReader for Option<T>
where
    T: Style,
{
    fn read_style(group: impl StyleGroup) -> Self {
        group.get_style()
    }
}

impl<T> StyleReader for T
where
    T: Style + Default,
{
    fn read_style(group: impl StyleGroup) -> Self {
        group.get_style().unwrap_or_default()
    }
}

#[cfg(doc)]
impl<T> StyleReader for (T,)
where
    T: StyleReader,
{
    fn read_style(group: &impl StyleGroup) -> Self {
        (T::read_style(group),)
    }
}

macro_rules! impl_reader {
    ($($($T:ident)*,)*) => {
        $(
            #[cfg(not(doc))]
            impl<$($T,)*> StyleReader for ($($T,)*)
            where
                $($T: StyleReader,)*
            {
                #[allow(clippy::unused_unit)]
                fn read_style(_container: impl StyleGroup) -> Self {
                    ($($T::read_style(&_container),)*)
                }
            }
        )*
    };
}

impl_reader! {
    ,
    A,
    A B,
    A B C,
    A B C D,
    A B C D E,
    A B C D E F,
    A B C D E F G,
    A B C D E F G H,
    A B C D E F G H I,
    A B C D E F G H I J,
    A B C D E F G H I J K,
    A B C D E F G H I J K L,
}

#[macro_export]
macro_rules! read_style {
    ($group: expr => {
        $($name:ident: $Style: ty,)*
    })=>{
        let ($($name,)*) = {
            let style_group = $group;
            ($(<$Style as $crate::style::reader::StyleReader>::read_style(style_group),)*)
        };
    };

    ($binding:ident in $group: expr => {
        $($name:ident: $Style: ty,)*
    }) => {
        let $binding = {
            struct __IrisiaAnonymousStyleReader {
                $($name: $Style,)*
            }

            let style_group = $group;

            __IrisiaAnonymousStyleReader {
                $($name: <$Style as $crate::style::reader::StyleReader>::read_style(style_group),)*
            }
        };
    };
}
