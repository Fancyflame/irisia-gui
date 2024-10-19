use std::rc::{Rc, Weak};

use super::{Listenable, ReadRef, ReadWire, Readable, ToListener};

pub fn const_wire<T: 'static>(data: T) -> ReadWire<T> {
    Rc::new_cyclic(|weak| ConstWire {
        data,
        this: weak.clone(),
    })
}

struct ConstWire<T> {
    data: T,
    this: Weak<Self>,
}

impl<T> Readable for ConstWire<T> {
    type Data = T;

    fn read(&self) -> ReadRef<Self::Data> {
        ReadRef::Ref(&self.data)
    }

    fn ptr_as_id(&self) -> *const () {
        self.this.as_ptr().cast()
    }
}

impl<T> Listenable for ConstWire<T> {
    fn add_listener(&self, _: &dyn ToListener) {}
    fn remove_listener(&self, _: &dyn ToListener) {}
}
