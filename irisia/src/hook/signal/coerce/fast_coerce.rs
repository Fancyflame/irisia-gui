use crate::{
    coerce_hook,
    hook::Signal,
    model::{VModel, control_flow::CommonVModel},
};

impl<T, Cd: 'static> From<Signal<T>> for Signal<dyn CommonVModel<Cd>>
where
    T: VModel<Cd> + 'static,
{
    fn from(value: Signal<T>) -> Self {
        coerce_hook!(value)
    }
}

impl<T> From<Signal<T>> for Signal<dyn AsRef<str>>
where
    T: AsRef<str> + 'static,
{
    fn from(value: Signal<T>) -> Self {
        coerce_hook!(value)
    }
}
