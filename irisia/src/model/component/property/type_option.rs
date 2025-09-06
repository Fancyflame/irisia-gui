#[derive(Default)]
pub struct TSome<T>(pub T);

#[derive(Default)]
pub struct TNone;

pub trait TypeOption<T> {
    type OrOutput<Other>;
    fn or<Other>(self, other: Other) -> Self::OrOutput<Other>;
    fn into_value(self) -> Option<T>;
}

impl<T> TypeOption<T> for TSome<T> {
    type OrOutput<Other> = Self;
    fn or<Other>(self, _: Other) -> Self {
        self
    }
    fn into_value(self) -> Option<T> {
        Some(self.0)
    }
}

impl<T> TypeOption<T> for TNone {
    type OrOutput<Other> = Other;
    fn or<Other>(self, other: Other) -> Other {
        other
    }
    fn into_value(self) -> Option<T> {
        None
    }
}
