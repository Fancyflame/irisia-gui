use crate::{
    coerce_hook,
    hook::Signal,
    model::{
        VModel, VNode,
        control_flow::common_vmodel::{DynVModel, DynVNode},
    },
};

mod coerce_inner;

pub trait SignalCast<T: ?Sized> {
    fn cast(from: Signal<Self>) -> Signal<T>;
}

macro_rules! cast {
    ($from:ty => $to:ty) => {
        fn cast(value: Signal<$from>) -> Signal<$to> {
            coerce_hook!(value)
        }
    };
}

impl<T> SignalCast<dyn AsRef<str>> for T
where
    T: AsRef<str> + 'static,
{
    cast!(T => dyn AsRef<str>);
}

impl<T, Data> SignalCast<DynVModel<Data>> for T
where
    T: VModel<Data> + 'static,
    Data: 'static,
{
    cast!(T => DynVModel<Data>);
}

impl<T, Data> SignalCast<DynVNode<Data>> for T
where
    T: VNode<Data> + 'static,
    Data: 'static,
{
    cast!(T => DynVNode<Data>);
}
