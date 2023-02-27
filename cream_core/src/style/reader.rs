use super::{Style, StyleContainer};

pub trait StyleReader {
    fn read_style<T: StyleContainer>(container: &T) -> Self;
}

impl<T> StyleReader for Option<T>
where
    T: Style,
{
    fn read_style<S: StyleContainer>(container: &S) -> Self {
        container.get_style()
    }
}

impl<T> StyleReader for T
where
    T: Style + Default,
{
    fn read_style<S: StyleContainer>(container: &S) -> Self {
        container.get_style().unwrap_or_default()
    }
}

macro_rules! impl_reader {
    ($($($T:ident)*,)*) => {
        $(
            impl<$($T,)*> StyleReader for ($($T,)*)
            where
                $($T: StyleReader,)*
            {
                fn read_style<Ctnr: StyleContainer>(_container: &Ctnr) -> Self {
                    ($($T::read_style(_container),)*)
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
