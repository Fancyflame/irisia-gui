use crate::style::StyleContainer;

pub trait SetStdStyles<'a, T>
where
    T: StyleContainer,
{
    type Output;
    fn set_std_styles(self, styles: &'a T) -> Self::Output;
}

impl<'a, T> SetStdStyles<'a, T> for ()
where
    T: StyleContainer,
{
    type Output = ();
    fn set_std_styles(self, _: &T) -> Self::Output {}
}
