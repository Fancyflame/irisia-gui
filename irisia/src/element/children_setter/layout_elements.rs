use std::marker::PhantomData;

use anyhow::anyhow;

use crate::{
    dom::{children::RenderMultiple, ElementModel},
    primitive::Region,
    structure::{Visit, VisitMut, Visitor, VisitorMut},
    style::StyleContainer,
    Element, Result, StyleReader,
};

#[must_use = "once you setted children, you must give them regions where \
they can draw content on. drop this struct unhandled WILL PANIC at runtime."]
pub struct LayoutElements<'a, T> {
    el: &'a mut T,
    applied: bool,
}

impl<'a, T> LayoutElements<'a, T> {
    pub(super) fn new(t: &'a mut T) -> Self {
        LayoutElements {
            el: t,
            applied: false,
        }
    }

    pub fn peek_styles<F, Sr>(&self, f: F)
    where
        F: FnMut(Sr),
        Sr: StyleReader,
        T: Visit<Vis<F, Sr>>,
    {
        let _ = self.el.visit(&mut Vis {
            visit: f,
            _sr: PhantomData,
        });
    }

    pub fn layout<F, Sr>(mut self, f: F) -> Result<()>
    where
        F: FnMut(Sr) -> Option<Region>,
        Sr: StyleReader,
        T: VisitMut<ApplyRegion<F, Sr>>,
    {
        self.applied = true;
        self.el.visit_mut(&mut ApplyRegion {
            provider: f,
            _sr: PhantomData,
        })
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

pub struct ApplyRegion<F, Sr> {
    provider: F,
    _sr: PhantomData<Sr>,
}

impl<El, Sty, Sc, F, Sr> VisitorMut<ElementModel<El, Sty, Sc>> for ApplyRegion<F, Sr>
where
    F: FnMut(Sr) -> Option<Region>,
    El: Element,
    Sc: RenderMultiple,
    Sty: StyleContainer,
    Sr: StyleReader,
{
    fn visit_mut(&mut self, data: &mut ElementModel<El, Sty, Sc>) -> Result<()> {
        match (self.provider)(data.styles().read()) {
            Some(region) => Ok(data.layout(region)),
            None => Err(anyhow!("regions provided is not enough")),
        }
    }
}

impl<T> Drop for LayoutElements<'_, T> {
    fn drop(&mut self) {
        assert!(self.applied, "`LayoutElements` must be used");
    }
}
