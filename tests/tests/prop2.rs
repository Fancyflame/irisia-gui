use std::{default, marker::PhantomData};

use irisia::Property;

// Start Prop

#[derive(Default)]
struct TSome<T>(pub T);

#[derive(Default)]
struct TNone;

trait StaticOption<T> {
    type OrOutput<Other>;
    fn or<Other>(self, other: Other) -> Self::OrOutput<Other>;

    fn unwrap_or_else(self, get_value: impl Fn() -> T) -> T;
}

impl<T> StaticOption<T> for TSome<T> {
    type OrOutput<Other> = Self;
    fn or<Other>(self, _: Other) -> Self {
        self
    }

    fn unwrap_or_else(self, _: impl Fn() -> T) -> T {
        self.0
    }
}

impl<T> StaticOption<T> for TNone {
    type OrOutput<Other> = Other;
    fn or<Other>(self, other: Other) -> Other {
        other
    }

    fn unwrap_or_else(self, get_value: impl Fn() -> T) -> T {
        get_value()
    }
}

trait MergeInto<From> {
    type Output;
    fn merge(self, other: From) -> Self::Output;
}

trait PropFrom<T> {
    fn prop_from(from: T) -> Self;
}

// Start Example

#[derive(Property)]
struct Foo<T: Sized, U>
where
    U: Sized,
{
    specific: u32,
    generic: Box<T>,
    optional_specific: u32,
    optional_generic: Option<U>,
}

struct FooTemplate<T, U, _1 = TNone, _2 = TNone, _3 = TNone, _4 = TNone> {
    _phantom: PhantomData<Foo<T, U>>,
    specific: _1,
    generic: _2,
    optional_specific: _3,
    optional_generic: _4,
}

impl<T, U> Default for FooTemplate<T, U> {
    fn default() -> Self {
        FooTemplate {
            _phantom: PhantomData,
            specific: TNone,
            generic: TNone,
            optional_specific: TNone,
            optional_generic: TNone,
        }
    }
}

impl<T, U, _S1, _S2, _S3, _S4, _O1, _O2, _O3, _O4> MergeInto<FooTemplate<T, U, _S1, _S2, _S3, _S4>>
    for FooTemplate<T, U, _O1, _O2, _O3, _O4>
where
    _O1: StaticOption<u32>,
    _O2: StaticOption<Box<T>>,
    _O3: StaticOption<u32>,
    _O4: StaticOption<Option<U>>,
{
    type Output = FooTemplate<
        T,
        U,
        _O1::OrOutput<_S1>,
        _O2::OrOutput<_S2>,
        _O3::OrOutput<_S3>,
        _O4::OrOutput<_S4>,
    >;

    #[rustfmt::skip]
    fn merge(self, other: FooTemplate<T, U, _S1, _S2, _S3, _S4>) -> Self::Output {
        FooTemplate {
            _phantom: PhantomData,
            specific: self.specific.or(other.specific),
            generic: self.generic.or(other.generic),
            optional_specific: self.optional_specific.or(other.optional_specific),
            optional_generic: self.optional_generic.or(other.optional_generic),
        }
    }
}

impl<T, U, _3, _4> FooTemplate<T, U, TSome<u32>, TSome<Box<T>>, _3, _4>
where
    _3: StaticOption<u32>,
    _4: StaticOption<Option<U>>,
{
    fn finish(self) -> Foo<T, U> {
        Foo {
            specific: self.specific.0,
            generic: self.generic.0,
            optional_specific: self.optional_specific.unwrap_or_else(Default::default),
            optional_generic: self.optional_generic.unwrap_or_else(Default::default),
        }
    }
}

struct FooBuilder<T, U>(PhantomData<Foo<T, U>>);

impl<T, U> FooBuilder<T, U> {
    const GET: Self = FooBuilder(PhantomData);
}

impl<T, U> FooBuilder<T, U> {
    fn specific(&self, value: u32) -> FooTemplate<T, U, TSome<u32>, TNone, TNone, TNone> {
        FooTemplate {
            _phantom: PhantomData,
            specific: TSome(value),
            generic: TNone,
            optional_specific: TNone,
            optional_generic: TNone,
        }
    }

    fn generic(&self, value: Box<T>) -> FooTemplate<T, U, TNone, TSome<Box<T>>, TNone, TNone> {
        FooTemplate {
            _phantom: PhantomData,
            specific: TNone,
            generic: TSome(value),
            optional_specific: TNone,
            optional_generic: TNone,
        }
    }

    fn optional_specific(&self, value: u32) -> FooTemplate<T, U, TNone, TNone, TSome<u32>, TNone> {
        FooTemplate {
            _phantom: PhantomData,
            specific: TNone,
            generic: TNone,
            optional_specific: TSome(value),
            optional_generic: TNone,
        }
    }

    fn optional_generic(
        &self,
        value: Option<U>,
    ) -> FooTemplate<T, U, TNone, TNone, TNone, TSome<Option<U>>> {
        FooTemplate {
            _phantom: PhantomData,
            specific: TNone,
            generic: TNone,
            optional_specific: TNone,
            optional_generic: TSome(value),
        }
    }
}

fn test() {
    let value = FooTemplate::default();
    let value = FooBuilder::GET.generic(Box::new(true)).merge(value);
    let value = FooBuilder::GET.optional_generic(Some("wow")).merge(value);
    let value = FooBuilder::GET.specific(10).merge(value);
    value.finish();
}
