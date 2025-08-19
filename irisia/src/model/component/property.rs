pub trait SetProperty<Name, Value>: GetPropertyType<Name> {
    type ReturnSelf;
    fn set(self, value: Value) -> Self::ReturnSelf;
}

pub trait GetPropertyType<Name> {
    type PropertyType;
}

pub type PropertyType<T, Name> = <T as GetPropertyType<Name>>::PropertyType;

pub fn set_property<T, V, Name>(body: T, value: V) -> T::ReturnSelf
where
    T: SetProperty<Name, V>,
{
    body.set(value)
}

pub struct PChar<const C: char>;

// struct Foo;

// impl SetProperty<(Char<'t'>, Char<'e'>, Char<'s'>, Char<'t'>)> for Foo {
//     type ReturnSelf = ();
//     type Value = ();
//     fn set(self, value: Self::Value) -> Self::ReturnSelf {}
// }
