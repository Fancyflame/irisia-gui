use std::fmt::Display;

use irisia::{
    Property, Signal,
    model::component::property::{PropCast, PropUpdate},
};

#[derive(Property, Debug)]
struct Foo<T, U: Display> {
    specific: Signal<u32>,
    generic: Signal<Box<T>>,
    optional_specific: Option<Signal<u32>>,
    optional_generic: Option<Signal<U>>,
    #[prop(extend)]
    bar: Bar,
}

#[derive(Property, Debug)]
struct Bar {
    #[prop(rename = "wawa")]
    a: Option<Signal<String>>,
}

macro_rules! init_prop {
    ($Type:ident {
        $($ident:ident: $value:expr,)*
    }) => {{
        let value = $Type::EMPTY;
        let mutator = $Type::MUTATOR;
        $(
            let value = $Type::prop_update(
                value,
                mutator.$ident($value),
            );
        )*
        value
    }};
}

fn main() {
    let foo = init_prop! {
        Foo {
            generic: Signal::state(Box::new(true)).to_read(),
            optional_generic: Signal::state("wow").to_read(),
            specific: Signal::state(10).to_read(),
            wawa: Signal::state("pig".into()).to_read(),
            // bar: init_prop! {
            //     Bar {
            //         // wawa: Signal::state("pig".into()).to_signal(),
            //     }
            // },
        }
    };
    let foo = Foo::prop_cast(foo);
}
