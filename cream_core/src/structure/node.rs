use crate::{element::RenderContent, primary::Region, style::reader::StyleReader, Result};

pub trait RenderingNode: Sized {
    type Cache: Default + Send + Sync + 'static;

    type StyleIter<'a, Item>: Iterator<Item = Item>
    where
        Self: 'a;

    type RegionIter<'a>: Iterator<Item = (Option<u32>, Option<u32>)>
    where
        Self: 'a;

    fn prepare_for_rendering(&mut self, cache: &mut Self::Cache, content: RenderContent);

    fn region_iter(&self) -> Self::RegionIter<'_>;

    fn style_iter<S>(&self) -> Self::StyleIter<'_, S>
    where
        S: StyleReader;

    fn finish<S, F>(
        self,
        cache: &mut Self::Cache,
        content: RenderContent,
        map: &mut F,
    ) -> Result<()>
    where
        F: FnMut(S, (Option<u32>, Option<u32>)) -> Result<Region>,
        S: StyleReader;
}
