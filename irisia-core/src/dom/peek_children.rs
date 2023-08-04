use std::marker::PhantomData;

use crate::{
    structure::{Visit, Visitor},
    style::StyleContainer,
    StyleReader,
};

use super::{ComputeSize, ElementModel};

#[derive(Clone)]
pub struct PeekChildren<'a, Ch>(&'a Ch);

impl<Ch> PeekChildren<'_, Ch> {
    pub fn peek<F, Sr>(&self, f: F)
    where
        F: FnMut(ComputeSize, Sr),
        Sr: StyleReader,
        Ch: Visit<Peeking<F, Sr>>,
    {
        let mut p = Peeking {
            f,
            _reader: PhantomData::<Sr>,
        };

        self.0.visit(&mut p);
    }
}

#[doc(hidden)]
pub struct Peeking<F, Sr> {
    f: F,
    _reader: PhantomData<Sr>,
}

impl<El, Sty, F, Sr> Visitor<ElementModel<El, Sty>> for Peeking<F, Sr>
where
    Sty: StyleContainer,
    F: FnMut(ComputeSize, Sr),
    Sr: StyleReader,
{
    fn visit(&mut self, data: &ElementModel<El, Sty>, _: &mut crate::structure::ControlFlow) {
        (self.f)(data.computed_size, data.styles.read());
    }
}
