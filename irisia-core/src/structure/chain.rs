use crate::{style::reader::StyleReader, Result};

use super::{node::BareContentWrapper, RenderingNode, VisitIter};

#[derive(Default)]
pub struct ChainCache<A, B>(pub(super) A, pub(super) B);

pub struct Chain<A, B>(pub(super) A, pub(super) B);

impl<A, B> RenderingNode for Chain<A, B>
where
    A: RenderingNode,
    B: RenderingNode,
{
    type Cache = ChainCache<A::Cache, B::Cache>;

    fn prepare_for_rendering(&mut self, cache: &mut Self::Cache, content: &BareContentWrapper) {
        self.0.prepare_for_rendering(&mut cache.0, content);
        self.1.prepare_for_rendering(&mut cache.1, content);
    }

    fn element_count(&self) -> usize {
        self.0.element_count() + self.1.element_count()
    }

    fn finish<S, F>(
        self,
        cache: &mut Self::Cache,
        mut content: BareContentWrapper,
        map: &mut F,
    ) -> crate::Result<()>
    where
        F: FnMut(S, (Option<u32>, Option<u32>)) -> Result<crate::primitive::Region>,
        S: StyleReader,
    {
        self.0.finish(
            &mut cache.0,
            BareContentWrapper(content.0.downgrade_lifetime()),
            map,
        )?;
        self.1.finish(&mut cache.1, content, map)
    }
}

impl<A, B, Prop> VisitIter<Prop> for Chain<A, B>
where
    A: VisitIter<Prop>,
    B: VisitIter<Prop>,
{
    type VisitIter<'a, S> =
        std::iter::Chain<A::VisitIter<'a, S>, B::VisitIter<'a, S>>
        where
            S:StyleReader,
            Self: 'a;

    fn visit_iter<S>(&self) -> Self::VisitIter<'_, S>
    where
        S: StyleReader,
    {
        self.0.visit_iter().chain(self.1.visit_iter())
    }
}
