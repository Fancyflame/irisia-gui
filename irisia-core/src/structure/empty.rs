use std::iter::Empty;

use crate::{style::reader::StyleReader, Result};

use super::{node::BareContentWrapper, RenderingNode, VisitItem, VisitIter};

#[derive(Clone, Copy)]
pub struct EmptyStructure;

impl RenderingNode for EmptyStructure {
    type Cache = ();

    fn prepare_for_rendering(&mut self, _: &mut Self::Cache, _: &BareContentWrapper) {}

    fn element_count(&self) -> usize {
        0
    }

    fn finish<S, F>(self, _: &mut Self::Cache, _: BareContentWrapper, _: &mut F) -> Result<()>
    where
        F: FnMut(S, (Option<u32>, Option<u32>)) -> Result<crate::primitive::Region>,
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
