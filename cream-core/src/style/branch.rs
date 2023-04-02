use super::*;

#[derive(Clone)]
pub enum Branch<T, U> {
    Arm1(T),
    Arm2(U),
}

impl<T, U> StyleContainer for Branch<T, U>
where
    T: StyleContainer,
    U: StyleContainer,
{
    fn get_style<S: Style>(&self) -> Option<S> {
        match self {
            Branch::Arm1(t) => t.get_style(),
            Branch::Arm2(f) => f.get_style(),
        }
    }
}
