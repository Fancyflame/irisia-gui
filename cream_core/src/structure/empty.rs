use std::iter::Empty;

use crate::{element::RenderContent, style::reader::StyleReader, Result};

use super::{RenderingNode, VisitItem, VisitIter};

#[derive(Clone, Copy)]
pub struct EmptyStructure;

impl RenderingNode for EmptyStructure {
    type Cache = ();

    fn prepare_for_rendering(&mut self, _: &mut Self::Cache, _: RenderContent) {}

    fn element_count(&self) -> usize {
        0
    }

    fn finish<S, F>(self, _: &mut Self::Cache, _: RenderContent, _: &mut F) -> Result<()>
    where
        F: FnMut(S, (Option<u32>, Option<u32>)) -> Result<crate::primary::Region>,
        S: StyleReader,
    {
        Ok(())
    }
}

impl<Prop> VisitIter<Prop> for EmptyStructure {
    type VisitIter<'a, S> = Empty<VisitItem<S, Prop>>
    where
        S:StyleReader
    ;

    fn visit_iter<S>(&self) -> Self::VisitIter<'_, S>
    where
        S: StyleReader,
    {
        std::iter::empty()
    }
}
