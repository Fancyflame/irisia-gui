use crate::{element::render_content::WildRenderContent, style::reader::StyleReader};

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
    type StyleIter<'a, S> =
        std::iter::Chain<<A as Node>::StyleIter<'a, S>, <B as Node>::StyleIter<'a, S>>
        where
            Self: 'a;

    fn style_iter<S>(&self) -> Self::StyleIter<'_, S>
    where
        S: StyleReader,
    {
        self.0.style_iter().chain(self.1.style_iter())
    }

    fn finish_iter<'a, I>(self, cache: &mut Self::Cache, mut iter: I) -> crate::Result<()>
    where
        I: Iterator<Item = WildRenderContent<'a>>,
    {
        self.0.finish_iter(&mut cache.0, &mut iter)?;
        self.1.finish_iter(&mut cache.1, iter)
    }
}
