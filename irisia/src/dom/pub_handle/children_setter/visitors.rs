use std::marker::PhantomData;

use anyhow::anyhow;

use crate::{
    dom::{children::RenderMultiple, ElementModel},
    primitive::Region,
    structure::{Visit, Visitor, VisitorMut},
    style::StyleContainer,
    Element, Result, StyleReader,
};

pub struct VisitStyles<F, Sr> {
    visit: F,
    _sr: PhantomData<Sr>,
}

impl<F, Sr> VisitStyles<F, Sr> {
    pub(super) fn new(visit: F) -> Self {
        VisitStyles {
            visit,
            _sr: PhantomData,
        }
    }
}

impl<El, Sty, Sc, F, Sr> Visitor<ElementModel<El, Sty, Sc>> for VisitStyles<F, Sr>
where
    F: FnMut(Sr),
    Sty: StyleContainer,
    Sr: StyleReader,
{
    fn visit(&mut self, data: &ElementModel<El, Sty, Sc>) -> Result<()> {
        (self.visit)(data.styles::<Sr>());
        Ok(())
    }
}

pub struct ApplyRegion<F, Sr> {
    provider: F,
    _sr: PhantomData<Sr>,
}

impl<F, Sr> ApplyRegion<F, Sr> {
    pub fn new(f: F) -> Self {
        ApplyRegion {
            provider: f,
            _sr: PhantomData,
        }
    }
}

impl<El, Sty, Sc, F, Sr> Visitor<ElementModel<El, Sty, Sc>> for ApplyRegion<F, Sr>
where
    F: FnMut(Sr) -> Option<Region>,
    El: Element,
    Sc: RenderMultiple,
    Sty: StyleContainer,
    Sr: StyleReader,
{
    fn visit(&mut self, data: &ElementModel<El, Sty, Sc>) -> Result<()> {
        match (self.provider)(data.styles()) {
            Some(region) => Ok(data.layout(region)),
            None => Err(anyhow!("regions provided is not enough")),
        }
    }
}
