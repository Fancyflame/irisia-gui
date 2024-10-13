use std::rc::Rc;

use super::{Listenable, ReadRef, ReadWire, Readable, ToListener};

pub fn const_wire<T: 'static>(data: T) -> ReadWire<T> {
    Rc::new(ConstWire(data))
}

struct ConstWire<T>(T);

impl<T> Readable for ConstWire<T> {
    type Data = T;

    fn read(&self) -> ReadRef<Self::Data> {
        ReadRef::Ref(&self.0)
    }
}

impl<T> Listenable for ConstWire<T> {
    fn add_listener(&self, _: &dyn ToListener) {}
    fn remove_listener(&self, _: &dyn ToListener) {}
}
