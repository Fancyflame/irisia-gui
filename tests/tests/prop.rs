use std::{default, fmt::Display, marker::PhantomData};

use irisia::{
    Property, Signal,
    model::component::property::{
        MergePropertiesFrom, PropCast, PropCreate, PropEmpty, PropUpdate, PropertyMutator,
    },
};

#[derive(Property, Debug)]
struct Foo<T, U: Display> {
    specific: Signal<u32>,
    generic: Signal<Box<T>>,
    optional_specific: Option<Signal<u32>>,
    optional_generic: Option<Signal<U>>,
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
        $(
            let value = $Type::prop_update(
                value,
                $Type::GET.$ident($value),
            );
        )*
        value
    }};
}

fn main() {
    let foo = init_prop! {
        Foo {
            generic: Signal::state(Box::new(true)).to_signal(),
            optional_generic: Signal::state("wow").to_signal(),
            specific: Signal::state(10).to_signal(),
            bar: init_prop! {
                Bar {
                    // wawa: Signal::state("pig".into()).to_signal(),
                }
            },
        }
    };
    let foo = Foo::prop_cast(foo);
}
