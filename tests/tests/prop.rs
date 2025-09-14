use std::{
    fmt::{Debug, Display},
    rc::Rc,
};

use irisia::{
    Property, Signal, coerce_hook,
    model::component::property::{
        PropCast, PropUpdate, PropertyAgent, macro_utils::coerce_signal_helper,
    },
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
}

macro_rules! init_prop {
    ($Type:ident {
        $($ident:ident: $value:expr,)*
    }) => {{
        let agent = $Type::__irisia_prop_agent();
        let value = agent.get_empty();
        $(
            let value = agent.prop_update(
                value,
                agent.$ident($value),
            );
        )*
        value
    }};
}

fn main() {
    // let foo = init_prop! {
    //     Foo {
    //         generic: Signal::state(Box::new(true)).to_read(),
    //         optional_generic: Signal::state("wow").to_read(),
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
        let agent = Foo::__irisia_prop_agent();
        let value = agent.get_empty();
        let value = agent.prop_update(
            value,
            agent.generic((Signal::state(Box::new(true)).to_read())),
        );
        let value = agent.prop_update(
            value,
            agent.optional_generic((Signal::state("wow").to_read())),
        );
        let value = agent.prop_update(value, agent.specific((Signal::state(10).to_read())));
        let value = agent.prop_update(
            value,
            agent.wawa(coerce_signal_helper(|sig| {
                agent.wawa(sig);
            })(coerce_hook!(Signal::state("pig").to_read()))),
        );
        value
    };

    let foo = Foo::__irisia_prop_agent().prop_cast(foo);
}

fn get_sig<T, F>(_: F) -> impl FnOnce(Signal<T>) -> Signal<T>
where
    T: ?Sized,
    F: FnOnce(Signal<T>),
{
    |sig| sig
}
