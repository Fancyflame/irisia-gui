use std::cell::RefMut;

use crate::{dom::ChildNodes, primitive::Region, style::StyleContainer, Result, StyleReader};

#[must_use]
pub struct LayoutElements<'a> {
    pub(super) refmut: RefMut<'a, dyn ChildNodes>,
}

impl<'a> LayoutElements<'a> {
    pub fn peek_styles<F, Sr>(&self, mut f: F)
    where
        F: FnMut(Sr),
        Sr: StyleReader,
    {
        self.refmut
            .peek_styles(&mut |inside_style_box| f(inside_style_box.read()));
    }

    pub fn len(&self) -> usize {
        self.refmut.len()
    }

    pub fn is_empty(&self) -> bool {
        self.refmut.len() == 0
    }

    pub fn layout<F, Sr>(self, mut layouter: F) -> Result<()>
    where
        F: FnMut(Sr) -> Option<Region>,
        Sr: StyleReader,
    {
        self.refmut
            .layout(&mut |inside_style_box| layouter(inside_style_box.read()))
    }

    pub fn layout_once(self, draw_region: Region) -> Result<()> {
        let mut dr = Some(draw_region);
        self.layout(|()| dr.take())
    }
}
