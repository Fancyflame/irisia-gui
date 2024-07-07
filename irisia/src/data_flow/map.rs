use std::rc::Rc;

use super::{ReadRef, ReadWire, Readable};

pub struct Map<T: ?Sized, F> {
    src: Rc<T>,
    map: F,
}

impl<T, F, Data1, Data2> Map<T, F>
where
    T: Readable<Data = Data1> + ?Sized,
    Data1: ?Sized,
    Data2: ?Sized,
    F: Fn(&Data1) -> &Data2,
{
    pub fn new(src: Rc<T>, map: F) -> Self {
        Self { src, map }
    }

    pub fn map<F2, Data3>(self, map2: F2) -> Map<T, MapPipe<F, F2>>
    where
        Data2: 'static,
        F2: Fn(&Data2) -> &Data3,
    {
        Map {
            src: self.src,
            map: MapPipe(self.map, map2),
        }
    }

    pub fn into_wire(self) -> ReadWire<Data2>
    where
        T: 'static,
        F: 'static,
    {
        Rc::new(self)
    }
}

impl<T, F, U> Readable for Map<T, F>
where
    T: Readable + ?Sized,
    F: MapFn<T::Data, Output = U>,
    U: ?Sized,
{
    type Data = U;

    fn read(&self) -> ReadRef<Self::Data> {
        ReadRef::map(self.src.read(), |r| self.map.map(r))
    }

    fn pipe(&self, listen_end: super::Listener) {
        self.src.pipe(listen_end)
    }
}

pub trait MapFn<T: ?Sized> {
    type Output: ?Sized;
    fn map<'a>(&self, data: &'a T) -> &'a Self::Output;
}

impl<F, T, R> MapFn<T> for F
where
    T: ?Sized,
    R: ?Sized,
    F: Fn(&T) -> &R,
{
    type Output = R;
    fn map<'a>(&self, data: &'a T) -> &'a Self::Output {
        self(data)
    }
}

pub struct MapPipe<F1, F2>(F1, F2);

impl<Data1, F1, Data2, F2, Data3> MapFn<Data1> for MapPipe<F1, F2>
where
    F1: MapFn<Data1, Output = Data2>,
    F2: MapFn<Data2, Output = Data3>,
    Data1: ?Sized,
    Data2: ?Sized + 'static,
    Data3: ?Sized,
{
    type Output = Data3;

    fn map<'a>(&self, data: &'a Data1) -> &'a Self::Output {
        let d2 = self.0.map(data);
        self.1.map(d2)
    }
}
