use std::{fmt::Display, marker::PhantomData};

use irisia::{
    element::deps::EmptyProps,
    hook::{state::State, Memo},
};

#[irisia::props]
struct Foo<'a, T: Display + ?Sized> {
    req1: u32,
    v: &'a T,

    #[props(default)]
    optional: String,

    #[props(default = "asdads".to_string())]
    b: String,

    #[props(skip)]
    _phantom: PhantomData<&'a T>,
}

fn main() {
    // only if all required fields initialized, the type will be `Foo<_>`
    let a: Foo<str> = Foo::empty_props().req1(10).v(State::new("b"));

    // initialize optional fields doesn't affect the type
    let _: Foo<str> = a.optional("a".to_string());

    let _: Foo<str> = Foo::empty_props()
        .v("text") // comment this will cause a compile error
        .optional(Memo::new(|()| format!("www"), ()))
        .req1(2);
}
