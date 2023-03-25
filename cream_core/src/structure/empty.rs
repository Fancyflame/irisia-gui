use std::iter::Empty;

use crate::{style::reader::StyleReader, Result};

use super::Node;

#[derive(Clone, Copy)]
pub struct EmptyStructure;

impl Node for EmptyStructure {
    type Cache = ();
    type StyleIter<'a, S> = Empty<S>;

    fn style_iter<S>(&self) -> Empty<S>
    where
        S: StyleReader,
    {
        std::iter::empty()
    }

    fn __finish_iter<S, F>(
        self,
        _: &mut Self::Cache,
        _: crate::element::render_content::WildRenderContent,
        _: &mut F,
    ) -> Result<()>
    where
        F: FnMut(S, Option<crate::primary::Region>) -> Result<crate::primary::Region>,
        S: StyleReader,
    {
        Ok(())
    }
}
