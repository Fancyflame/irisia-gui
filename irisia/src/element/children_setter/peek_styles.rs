use std::marker::PhantomData;

use crate::{
    dom::ElementModel,
    structure::{Visit, Visitor},
    style::StyleContainer,
    Result, StyleReader,
};

pub struct PeekStyles<'a, T>(&'a T);

impl<'a, T> PeekStyles<'a, T> {
    pub(super) fn new(t: &'a T) -> Self {
        PeekStyles(t)
    }

    pub fn peek<F, Sr>(&self, f: F)
    where
        F: FnMut(Sr),
        Sr: StyleReader,
        T: Visit<Vis<F, Sr>>,
    {
        let _ = self.0.visit(&mut Vis {
            visit: f,
            _sr: PhantomData,
        });
    }
}

pub struct Vis<F, Sr> {
    visit: F,
    _sr: PhantomData<Sr>,
}

impl<El, Sty, Sc, F, Sr> Visitor<ElementModel<El, Sty, Sc>> for Vis<F, Sr>
where
    F: FnMut(Sr),
    Sty: StyleContainer,
    Sr: StyleReader,
{
    fn visit(&mut self, data: &ElementModel<El, Sty, Sc>) -> Result<()> {
        (self.visit)(data.styles().read());
        Ok(())
    }
}
