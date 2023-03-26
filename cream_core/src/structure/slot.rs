use crate::{primary::Region, style::reader::StyleReader, Result};

use super::*;

impl<'a, T> RenderingNode for Slot<'a, T>
where
    T: RenderingNode,
{
    type Cache = ();
    type StyleIter<'b, S> = T::StyleIter<'b, S>
    where
        Self: 'b;

    type RegionIter<'b> = T::RegionIter<'b>
    where
        Self:'b;

    fn prepare_for_rendering(&mut self, _: &mut Self::Cache, content: RenderContent) {
        self.node.prepare_for_rendering(self.cache, content);
    }

    fn style_iter<S>(&self) -> Self::StyleIter<'_, S>
    where
        S: StyleReader,
    {
        self.node.style_iter()
    }

    fn region_iter(&self) -> Self::RegionIter<'_> {
        self.node.region_iter()
    }

    fn finish<S, F>(self, _: &mut (), content: RenderContent, map: &mut F) -> Result<()>
    where
        F: FnMut(S, (Option<u32>, Option<u32>)) -> Result<Region>,
        S: StyleReader,
    {
        self.finish(content, map)
    }
}

pub struct Slot<'a, T>
where
    T: RenderingNode,
{
    pub(crate) node: T,
    pub(crate) cache: &'a mut T::Cache,
}

impl<'a, T> Slot<'a, T>
where
    T: RenderingNode,
{
    pub fn style_iter<S>(&self) -> T::StyleIter<'_, S>
    where
        S: StyleReader,
    {
        self.node.style_iter()
    }

    fn finish<S, F>(self, content: RenderContent, map: &mut F) -> Result<()>
    where
        F: FnMut(S, (Option<u32>, Option<u32>)) -> Result<Region>,
        S: StyleReader,
    {
        self.node.finish(self.cache, content, map)
    }
}
