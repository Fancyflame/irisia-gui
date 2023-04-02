use crate::{primary::Region, style::reader::StyleReader, Result};

use super::*;

pub struct Slot<'a, T>
where
    T: RenderingNode,
{
    pub(crate) node: T,
    pub(crate) cache: &'a mut T::Cache,
}

impl<'a, T> RenderingNode for Slot<'a, T>
where
    T: RenderingNode,
{
    type Cache = ();

    fn prepare_for_rendering(&mut self, _: &mut Self::Cache, content: RenderContent) {
        self.node.prepare_for_rendering(self.cache, content);
    }

    fn element_count(&self) -> usize {
        self.node.element_count()
    }

    fn finish<S, F>(self, _: &mut (), content: RenderContent, map: &mut F) -> Result<()>
    where
        F: FnMut(S, (Option<u32>, Option<u32>)) -> Result<Region>,
        S: StyleReader,
    {
        self.node.finish(self.cache, content, map)
    }
}

impl<'a, T, Prop> VisitIter<Prop> for Slot<'a, T>
where
    T: VisitIter<Prop>,
{
    type VisitIter<'b, S> = T::VisitIter<'b, S>
    where
        S:StyleReader,
        Self: 'b;

    fn visit_iter<S>(&self) -> Self::VisitIter<'_, S>
    where
        S: StyleReader,
    {
        self.node.visit_iter()
    }
}
