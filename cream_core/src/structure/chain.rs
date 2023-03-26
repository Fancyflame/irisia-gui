use crate::{style::reader::StyleReader, Result};

use super::Node;

#[derive(Default)]
pub struct ChainCache<A, B>(pub(super) A, pub(super) B);

pub struct Chain<A, B>(pub(super) A, pub(super) B);

impl<A, B> Node for Chain<A, B>
where
    A: Node,
    B: Node,
{
    type Cache = ChainCache<<A as Node>::Cache, <B as Node>::Cache>;
    type Iter<'a, S> =
        std::iter::Chain<<A as Node>::Iter<'a, S>, <B as Node>::Iter<'a, S>>
        where
            Self: 'a;

    fn style_iter<S>(&self) -> Self::Iter<'_, S>
    where
        S: StyleReader,
    {
        self.0.style_iter().chain(self.1.style_iter())
    }

    fn __finish_iter<S, F>(
        self,
        cache: &mut Self::Cache,
        mut content: crate::element::render_content::WildRenderContent,
        map: &mut F,
    ) -> crate::Result<()>
    where
        F: FnMut(S, Option<crate::primary::Region>) -> Result<crate::primary::Region>,
        S: StyleReader,
    {
        self.0
            .__finish_iter(&mut cache.0, content.downgrade_lifetime(), map)?;
        self.1.__finish_iter(&mut cache.1, content, map)
    }
}
