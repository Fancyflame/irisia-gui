use crate::style::StyleContainer;

pub trait SetStdStyles<T>
where
    T: StyleContainer,
{
    type Output;
    fn set_std_styles(self, styles: T) -> Self::Output;
}

impl<T> SetStdStyles<T> for ()
where
    T: StyleContainer,
{
    type Output = ();
    fn set_std_styles(self, _: T) -> Self::Output {}
}
