use crate::{element::RenderContent, primary::Region, style::reader::StyleReader, Result};

pub trait RenderingNode: Sized {
    type Cache: Default + Send + Sync + 'static;

    fn element_count(&self) -> usize;

    fn prepare_for_rendering(&mut self, cache: &mut Self::Cache, content: RenderContent);

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
