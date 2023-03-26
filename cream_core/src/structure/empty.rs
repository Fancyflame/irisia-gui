use std::iter::Empty;

use crate::{element::RenderContent, style::reader::StyleReader, Result};

use super::RenderingNode;

#[derive(Clone, Copy)]
pub struct EmptyStructure;

impl RenderingNode for EmptyStructure {
    type Cache = ();
    type StyleIter<'a, S> = Empty<S>;
    type RegionIter<'a> = Empty<(Option<u32>, Option<u32>)>;

    fn prepare_for_rendering(&mut self, _: &mut Self::Cache, _: RenderContent) {}

    fn style_iter<S>(&self) -> Empty<S>
    where
        S: StyleReader,
    {
        std::iter::empty()
    }

    fn region_iter(&self) -> Self::RegionIter<'_> {
        std::iter::empty()
    }

    fn finish<S, F>(self, _: &mut Self::Cache, _: RenderContent, _: &mut F) -> Result<()>
    where
        F: FnMut(S, (Option<u32>, Option<u32>)) -> Result<crate::primary::Region>,
        S: StyleReader,
    {
        Ok(())
    }
}
