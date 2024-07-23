use std::rc::Rc;

use super::{ReadRef, ReadWire, Readable};

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

    pub fn into_wire(self) -> ReadWire<Data2>
    where
        Data1: 'static,
        Data2: 'static,
        T: 'static,
    {
        Rc::new(self)
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

    fn pipe(&self, listen_end: super::Listener) {
        self.src.pipe(listen_end)
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
