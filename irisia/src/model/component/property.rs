pub trait Property<Name> {
    type ReturnSelf;
    type Value;
    fn set(self, value: Self::Value) -> Self::ReturnSelf;
}

pub struct PChar<const C: char>;

// struct Foo;

// impl SetProperty<(Char<'t'>, Char<'e'>, Char<'s'>, Char<'t'>)> for Foo {
//     type ReturnSelf = ();
//     type Value = ();
//     fn set(self, value: Self::Value) -> Self::ReturnSelf {}
// }
