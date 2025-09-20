use std::{default, marker::PhantomData, ops::BitOr};

use irisia::{
    __private::property::modname::ExtendHelper,
    Property, Signal,
    model::component::{
        definition::{Definition, DirectAssign, SignalProxied},
        property::{
            PropAssign, PropExtend, PropertyAgent,
            macro_utils::{EmptyOf, get_empty},
        },
    },
};

// Start Prop

struct TSome<T>(pub T);

struct TNone;

trait StaticOption {
    type OrOutput<Other>;
    fn or<Other>(self, other: Other) -> Self::OrOutput<Other>;
}

impl<T> StaticOption for TSome<T> {
    type OrOutput<Other> = T;
    fn or<Other>(self, _: Other) -> T {
        self.0
    }
}

impl StaticOption for TNone {
    type OrOutput<Other> = Other;
    fn or<Other>(self, other: Other) -> Other {
        other
    }
}

trait PropFrom<T> {
    fn prop_from(from: T) -> Self;
}

trait PropAgent {
    type Empty;
    fn get_empty(&self) -> Self::Empty;
}

// Start Example

struct Foo<T: Sized, U>
where
    U: Sized,
{
    specific: Signal<u32>,
    generic: Signal<Box<T>>,
    optional_specific: Option<Signal<u32>>,
    optional_generic: Option<Signal<U>>,
}

impl<T: Sized, U> Property for Foo<T, U>
where
    U: Sized,
{
    type Agent = FooAgent<T, U>;
    fn __irisia_prop_agent<'a>() -> &'a Self::Agent {
        &FooAgent(PhantomData)
    }
}

struct FooTemplate<T, U, _1, _2, _3, _4> {
    _phantom: PhantomData<Foo<T, U>>,
    specific: _1,
    generic: _2,
    optional_specific: _3,
    optional_generic: _4,
}

impl<T: Sized, U, _S1, _S2, _S3, _S4, _O1, _O2, _O3, _O4>
    BitOr<FooTemplate<T, U, _O1, _O2, _O3, _O4>> for FooTemplate<T, U, _S1, _S2, _S3, _S4>
where
    U: Sized,
    _S1: StaticOption,
    _S2: StaticOption,
    _S3: StaticOption,
    _S4: StaticOption,
{
    type Output = FooTemplate<
        T,
        U,
        _S1::OrOutput<_O1>,
        _S2::OrOutput<_O2>,
        _S3::OrOutput<_O3>,
        _S4::OrOutput<_O4>,
    >;

    #[rustfmt::skip]
    fn bitor(self, other: FooTemplate<T, U, _O1, _O2, _O3, _O4>) -> Self::Output {
        FooTemplate {
            _phantom: PhantomData,
            specific: self.specific.or(other.specific),
            generic: self.generic.or(other.generic),
            optional_specific: self.optional_specific.or(other.optional_specific),
            optional_generic: self.optional_generic.or(other.optional_generic),
        }
    }
}

impl<T, U, _1, _2, _3, _4> Definition for FooTemplate<T, U, _1, _2, _3, _4>
where
    Foo<T, U>: 'static,
    _1: Definition<Value = Signal<u32>>,
    _2: Definition<Value = Signal<Box<T>>>,
    _3: Definition<Value = Option<Signal<u32>>>,
    _4: Definition<Value = Option<Signal<U>>>,
{
    type Storage = FooTemplate<T, U, _1::Storage, _2::Storage, _3::Storage, _4::Storage>;
    type Value = Foo<T, U>;

    fn create(&self) -> (Self::Storage, Self::Value) {
        let specific = self.specific.create();
        let generic = self.generic.create();
        let optional_specific = self.optional_specific.create();
        let optional_generic = self.optional_generic.create();
        (
            FooTemplate {
                _phantom: PhantomData,
                specific: specific.0,
                generic: generic.0,
                optional_specific: optional_specific.0,
                optional_generic: optional_generic.0,
            },
            Foo {
                specific: specific.1,
                generic: generic.1,
                optional_specific: optional_specific.1,
                optional_generic: optional_generic.1,
            },
        )
    }

    fn update(&self, storage: &mut Self::Storage) {
        self.specific.update(&mut storage.specific);
        self.generic.update(&mut storage.generic);
        self.optional_specific
            .update(&mut storage.optional_specific);
        self.optional_generic.update(&mut storage.optional_generic);
    }
}

struct FooAgent<T, U>(PhantomData<Foo<T, U>>);

impl<T, U> PropertyAgent for FooAgent<T, U> {
    type CastTarget = Foo<T, U>;
    type Empty = FooTemplate<
        T,
        U,
        EmptyOf<Signal<u32>>,
        EmptyOf<Signal<Box<T>>>,
        EmptyOf<Option<Signal<u32>>>,
        EmptyOf<Option<Signal<U>>>,
    >;

    fn get_empty(&self) -> Self::Empty {
        FooTemplate {
            _phantom: PhantomData,
            specific: get_empty::<Signal<u32>>(),
            generic: get_empty::<Signal<Box<T>>>(),
            optional_specific: get_empty::<Option<Signal<u32>>>(),
            optional_generic: get_empty::<Option<Signal<U>>>(),
        }
    }
}

impl<T, U, _1, _2, _3, _4> FooTemplate<T, U, _1, _2, _3, _4> {
    fn specific<__IrisiaValue>(
        &self,
        value: __IrisiaValue,
    ) -> modname::ExtendHelper<Self, __IrisiaValue, FooTemplate<T, U, __IrisiaValue, _2, _3, _4>>
    where
        __IrisiaValue: Definition,
        Signal<u32>: PropAssign<__IrisiaValue>,
    {
        modname::ExtendHelper::new(value, |this, value| FooTemplate {
            _phantom: PhantomData,
            specific: value,
            generic: this.generic,
            optional_specific: this.optional_specific,
            optional_generic: this.optional_generic,
        })
    }
}

impl<T, U, _1, _2, _3, _4> PropExtend<Self> for FooTemplate<T, U, _1, _2, _3, _4> {
    type Output<Ext> = Ext;
    fn prop_extend<F, Ext>(self, f: F) -> Ext
    where
        F: FnOnce(Self) -> Ext,
    {
        f(self)
    }
}

impl<T, U> FooAgent<T, U> {
    fn generic<__IrisiaValue>(
        &self,
        value: __IrisiaValue,
    ) -> FooTemplate<T, U, TNone, TSome<__IrisiaValue>, TNone, TNone>
    where
        __IrisiaValue: Definition,
        Signal<Box<T>>: PropAssign<__IrisiaValue>,
    {
        FooTemplate {
            _phantom: PhantomData,
            specific: TNone,
            generic: TSome(value),
            optional_specific: TNone,
            optional_generic: TNone,
        }
    }

    fn optional_specific<__IrisiaValue>(
        &self,
        value: __IrisiaValue,
    ) -> FooTemplate<T, U, TNone, TNone, TSome<__IrisiaValue>, TNone>
    where
        __IrisiaValue: Definition,
        Option<Signal<u32>>: PropAssign<__IrisiaValue>,
    {
        FooTemplate {
            _phantom: PhantomData,
            specific: TNone,
            generic: TNone,
            optional_specific: TSome(value),
            optional_generic: TNone,
        }
    }

    fn optional_generic<__IrisiaValue>(
        &self,
        value: __IrisiaValue,
    ) -> FooTemplate<T, U, TNone, TNone, TNone, TSome<__IrisiaValue>>
    where
        __IrisiaValue: Definition,
        Option<Signal<Box<T>>>: PropAssign<__IrisiaValue>,
    {
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
    // let agent = Foo::__irisia_prop_agent();
    // let value = agent.get_empty();
    // let value = agent.specific(10) | value;
    // let value = agent.generic(Box::new(true)) | value;
    // let value = agent.optional_generic(Some("wow")) | value;
    // let value = agent.specific(10) | value;
    // value.create();
}
