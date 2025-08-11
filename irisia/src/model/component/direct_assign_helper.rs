use std::marker::PhantomData;

use super::definition::DirectAssign;

pub fn type_infer<F, Arg, T>(_: F) -> TypeInfer<T>
where
    F: FnOnce(Arg) -> Option<T>,
    T: Clone,
{
    TypeInfer(PhantomData)
}

pub struct TypeInfer<T>(PhantomData<T>);

impl<T: Clone> TypeInfer<T> {
    pub fn infer(&self, value: T) -> DirectAssign<T> {
        DirectAssign(value)
    }
}

#[test]
fn test() {
    use crate::{coerce_hook, hook::Signal};
    use std::fmt::Display;

    struct Props {
        foo: Option<Signal<dyn Display>>,
    }

    let string = Signal::state(12345);
    let casted: _ = type_infer(|v: Props| v.foo).infer(coerce_hook!(string.to_signal()));
    let _: DirectAssign<Signal<dyn Display>> = casted;
}
