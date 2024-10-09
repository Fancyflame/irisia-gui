use std::rc::Rc;

use super::{deps::Listener, Listenable, ReadRef, ReadWire, Readable};

pub struct Map<T, Data1, Data2>
where
    Data1: ?Sized,
    Data2: ?Sized,
{
    src: T,
    map: fn(&Data1) -> &Data2,
}

impl<T, Data1, Data2> Map<T, Data1, Data2>
where
    T: Readable<Data = Data1>,
    Data1: ?Sized,
    Data2: ?Sized,
{
    pub fn new(src: T, map: fn(&Data1) -> &Data2) -> Self {
        Self { src, map }
    }
}

impl<T, Data1, Data2> Readable for Map<T, Data1, Data2>
where
    T: Readable<Data = Data1>,
    Data1: ?Sized,
    Data2: ?Sized,
{
    type Data = Data2;

    fn read(&self) -> ReadRef<Self::Data> {
        ReadRef::map(self.src.read(), &self.map)
    }
}

impl<T, Data1, Data2> Listenable for Map<T, Data1, Data2>
where
    T: Listenable,
    Data1: ?Sized,
    Data2: ?Sized,
{
    fn add_listener(&self, listener: &Listener) {
        self.src.add_listener(listener);
    }

    fn remove_listener(&self, listener: &Listener) {
        self.src.remove_listener(listener);
    }
}

impl<T, Data1, Data2> Clone for Map<T, Data1, Data2>
where
    T: Clone,
    Data1: ?Sized,
    Data2: ?Sized,
{
    fn clone(&self) -> Self {
        Self {
            src: self.src.clone(),
            map: self.map,
        }
    }
}

impl<T, Data1, Data2> From<Map<T, Data1, Data2>> for ReadWire<Data2>
where
    T: Readable<Data = Data1>,
    Data1: ?Sized,
    Data2: ?Sized,
{
    fn from(value: Map<T, Data1, Data2>) -> Self {
        Rc::new(value)
    }
}
