#[derive(Default)]
pub struct TSome<T>(pub T);

#[derive(Default)]
pub struct TNone;

pub trait TypeOption<T> {
    type OrOutput<Other>;
    fn or<Other>(self, other: Other) -> Self::OrOutput<Other>;

    fn unwrap_or_else(self, get_value: impl Fn() -> T) -> T;
}

impl<T> TypeOption<T> for TSome<T> {
    type OrOutput<Other> = Self;
    fn or<Other>(self, _: Other) -> Self {
        self
    }

    fn unwrap_or_else(self, _: impl Fn() -> T) -> T {
        self.0
    }
}

impl<T> TypeOption<T> for TNone {
    type OrOutput<Other> = Other;
    fn or<Other>(self, other: Other) -> Other {
        other
    }

    fn unwrap_or_else(self, get_value: impl Fn() -> T) -> T {
        get_value()
    }
}
