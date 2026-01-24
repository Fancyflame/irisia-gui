use crate::{
    coerce_hook,
    hook::Signal,
    model::control_flow::general::{GeneralUnitVModel, GeneralVModel},
};

mod coerce_inner;

pub trait SignalCast<T: ?Sized> {
    fn cast(from: Signal<Self>) -> Signal<T>;
}

macro_rules! trait_cast {
    ($($Trait:tt)*) => {
        impl<T> SignalCast<dyn $($Trait)*> for T
        where
            T: $($Trait)* + 'static,
        {
            fn cast(from: Signal<Self>) -> Signal<dyn $($Trait)*> {
                coerce_hook!(from)
            }
        }
    }
}

trait_cast!(AsRef<str>);
trait_cast!(GeneralVModel);
trait_cast!(GeneralUnitVModel);
