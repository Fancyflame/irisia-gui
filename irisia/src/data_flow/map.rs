use std::{cell::Ref, rc::Rc};

use super::{ReadWire, Readable};

pub struct Map<T: ?Sized, F> {
    src: Rc<T>,
    map: F,
}

impl<T: ?Sized, F> Map<T, F> {
    pub fn new<U>(src: Rc<T>, map: F) -> ReadWire<U>
    where
        T: Readable + 'static,
        F: Fn(&T::Data) -> &U + 'static,
    {
        Rc::new(Self { src, map })
    }
}

impl<T, F, U> Readable for Map<T, F>
where
    T: Readable + ?Sized,
    F: Fn(&T::Data) -> &U,
{
    type Data = U;

    fn read(&self) -> std::cell::Ref<Self::Data> {
        Ref::map(self.src.read(), &self.map)
    }

    fn pipe(&self, listen_end: super::Listener) {
        self.src.pipe(listen_end)
    }
}
