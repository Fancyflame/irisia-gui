use std::{
    fmt::{Debug, Display},
    rc::Rc,
};

use irisia::{
    __private::new_proxy_signal,
    Property, Signal, coerce_hook,
    model::component::definition::{Definition, DirectAssign, SignalProxied},
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
    a: Option<Signal<dyn Debug>>,

    #[prop(extend)]
    baz: Baz,
}

#[derive(Property, Debug)]
struct Baz {
    baz_desc: Signal<dyn Debug>,
}

macro_rules! init_prop {
    ($Type:ident {
        $($ident:ident: $expr:expr,)*
    }) => {{
        let agent = $Type::__irisia_prop_agent();
        let value = agent.get_empty();
        $(
            let value = value.$ident(
                DirectAssign($expr)
            ).apply(value);
        )*
        value
    }};
}

fn main() {
    // let foo = init_prop! {
    //     Foo {
    //         generic: Signal::state(Box::new(true)).to_read(),
    //         optional_generic: coerce_hook!(Signal::state("wow").to_read()),
    //         specific: Signal::state(10).to_read(),
    //         wawa: {
    //             let mut rc = None;
    //             if false {
    //                 Foo::__irisia_prop_agent().wawa(Signal::__irisia_from_inner(rc.clone().unwrap()));
    //             }
    //             let rc = rc.insert((Signal::state("pig").to_read()).__irisia_to_inner() as _);
    //             Signal::__irisia_from_inner(rc.clone())

    //         },
    //         // bar: init_prop! {
    //         //     Bar {
    //         //         // wawa: Signal::state("pig".into()).to_signal(),
    //         //     }
    //         // },
    //     }
    // };
    let foo = {
        let value = Foo::__IRISIA_EMPTY_PROP;
        let value = value
            .generic(DirectAssign((Signal::state(Box::new(true)).to_read())))
            .apply(value);
        let value = value
            .optional_generic(new_proxy_signal("waw").get())
            .apply(value);
        let value = value
            .specific(DirectAssign((Signal::state(10).to_read())))
            .apply(value);

        let value = value
            .wawa(new_proxy_signal("pig").get().coerce_unsize_helped(|x| {
                value.wawa(x);
            })(|x| coerce_hook!(x)))
            .apply(value);

        let value = value
            .baz_desc(new_proxy_signal("hello").get().coerce_unsize_helped(|x| {
                value.baz_desc(x);
            })(|x| coerce_hook!(x)))
            .apply(value);
        value
    };
    let (_, foo) = Definition::create(&foo);
    dbg!(foo);

    // let foo = Foo::__irisia_prop_agent().prop_cast(foo);
}
