use crate::{element::RenderContent, style::reader::StyleReader, Result};

use super::RenderingNode;

#[derive(Default)]
pub struct ChainCache<A, B>(pub(super) A, pub(super) B);

pub struct Chain<A, B>(pub(super) A, pub(super) B);

impl<A, B> RenderingNode for Chain<A, B>
where
    A: RenderingNode,
    B: RenderingNode,
{
    type Cache = ChainCache<A::Cache, B::Cache>;
    type StyleIter<'a, S> =
        std::iter::Chain<A::StyleIter<'a, S>, B::StyleIter<'a, S>>
        where
            Self: 'a;

    type RegionIter<'a> = std::iter::Chain<A::RegionIter<'a>, B::RegionIter<'a>>
        where
            Self: 'a;

    fn prepare_for_rendering(&mut self, cache: &mut Self::Cache, mut content: RenderContent) {
        self.0
            .prepare_for_rendering(&mut cache.0, content.downgrade_lifetime());
        self.1.prepare_for_rendering(&mut cache.1, content)
    }

    fn style_iter<S>(&self) -> Self::StyleIter<'_, S>
    where
        S: StyleReader,
    {
        self.0.style_iter().chain(self.1.style_iter())
    }

    fn region_iter(&self) -> Self::RegionIter<'_> {
        self.0.region_iter().chain(self.1.region_iter())
    }

    fn finish<S, F>(
        self,
        cache: &mut Self::Cache,
        mut content: RenderContent,
        map: &mut F,
    ) -> crate::Result<()>
    where
        F: FnMut(S, (Option<u32>, Option<u32>)) -> Result<crate::primary::Region>,
        S: StyleReader,
    {
        self.0
            .finish(&mut cache.0, content.downgrade_lifetime(), map)?;
        self.1.finish(&mut cache.1, content, map)
    }
}
