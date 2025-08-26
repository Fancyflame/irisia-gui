use std::{default, marker::PhantomData};

#[derive(Default)]
struct TSome<T>(pub T);

struct TNone;

struct Foo<T, U = Option<&'static str>> {
    specific: u32,
    generic: Box<T>,
    optional_specific: u32,
    optional_generic: Option<U>,
}

struct FooProps<T, U, _1 = TSome<u32>, _2 = TSome<Box<T>>> {
    _phantom: PhantomData<Foo<T, U>>,
    specific: _1,
    generic: _2,
    optional_specific: u32,
    optional_generic: Option<U>,
}

impl<T, U> Default for FooProps<T, U, TNone, TNone> {
    fn default() -> Self {
        FooProps {
            _phantom: PhantomData,
            specific: TNone,
            generic: TNone,
            optional_specific: Default::default(),
            optional_generic: Default::default(),
        }
    }
}

impl<T, U> FooProps<T, U> {
    fn _finish(self) -> Foo<T, U> {
        Foo {
            specific: self.specific.0,
            generic: self.generic.0,
            optional_specific: self.optional_specific,
            optional_generic: self.optional_generic,
        }
    }
}

impl<T, U, _1, _2> FooProps<T, U, _1, _2> {
    fn specific(self, value: u32) -> FooProps<T, U, TSome<u32>, _2> {
        FooProps {
            _phantom: PhantomData,
            specific: TSome(value),
            generic: self.generic,
            optional_specific: self.optional_specific,
            optional_generic: self.optional_generic,
        }
    }

    fn generic(self, value: Box<T>) -> FooProps<T, U, _1, TSome<Box<T>>> {
        FooProps {
            _phantom: PhantomData,
            specific: self.specific,
            generic: TSome(value),
            optional_specific: self.optional_specific,
            optional_generic: self.optional_generic,
        }
    }

    fn optional_specific(self, value: u32) -> FooProps<T, U, _1, _2> {
        FooProps {
            _phantom: PhantomData,
            specific: self.specific,
            generic: self.generic,
            optional_specific: value,
            optional_generic: self.optional_generic,
        }
    }

    fn optional_generic(self, value: Option<U>) -> FooProps<T, U, _1, _2> {
        FooProps {
            _phantom: PhantomData,
            specific: self.specific,
            generic: self.generic,
            optional_specific: self.optional_specific,
            optional_generic: value,
        }
    }
}

fn test() {
    let props = FooProps::default()
        .generic(Box::new(true))
        .optional_generic(Some("wow"))
        .specific(10)
        ._finish();
}
