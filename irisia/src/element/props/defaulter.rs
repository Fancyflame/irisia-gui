use std::marker::PhantomData;

pub trait Defaulter {
    type Src;

    fn with_defaulter<F>(self, defaulter: F) -> Self::Src
    where
        F: FnOnce() -> Self::Src;
}

#[derive(Clone, Copy)]
pub struct PropNotInitialized<S>(pub(super) PhantomData<S>);

impl<S> Defaulter for PropNotInitialized<S> {
    type Src = S;
    fn with_defaulter<F>(self, defaulter: F) -> Self::Src
    where
        F: FnOnce() -> Self::Src,
    {
        defaulter()
    }
}

pub struct PropInitialized<S>(pub S);

impl<S> Defaulter for PropInitialized<S> {
    type Src = S;
    fn with_defaulter<F>(self, _: F) -> Self::Src
    where
        F: FnOnce() -> Self::Src,
    {
        self.0
    }
}

impl<S> PropInitialized<S> {
    pub fn must_be_initialized(self) -> S {
        self.0
    }
}
