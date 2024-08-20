use std::rc::Rc;

use super::{Listener, ReadRef, ReadWire, Readable};

struct ConstWire<T, F> {
    data: T,
    map: F,
}

impl<T, F, U> Readable for ConstWire<T, F>
where
    F: Fn(&T) -> &U,
    U: ?Sized,
{
    type Data = U;
    fn pipe(&self, _: Listener) {}
    fn read(&self) -> ReadRef<Self::Data> {
        ReadRef::Ref((self.map)(&self.data))
    }
}

pub fn const_wire<T: 'static>(data: T) -> ReadWire<T> {
    const_wire_unsized(data, |x: &T| x)
}

pub fn const_wire_unsized<T, F, U>(data: T, map: F) -> ReadWire<U>
where
    T: 'static,
    F: Fn(&T) -> &U + 'static,
    U: ?Sized,
{
    Rc::new(ConstWire { data, map })
}
