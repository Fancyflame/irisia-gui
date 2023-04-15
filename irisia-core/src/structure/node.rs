use crate::{
    element::render_content::BareContent, primary::Region, style::reader::StyleReader, Result,
};

pub struct BareContentWrapper<'a>(pub(crate) BareContent<'a>);

pub trait RenderingNode: Sized {
    type Cache: Default + 'static;

    fn element_count(&self) -> usize;

    fn prepare_for_rendering(&mut self, cache: &mut Self::Cache, content: &BareContentWrapper);

    fn finish<S, F>(
        self,
        cache: &mut Self::Cache,
        content: BareContentWrapper,
        map: &mut F,
    ) -> Result<()>
    where
        F: FnMut(S, (Option<u32>, Option<u32>)) -> Result<Region>,
        S: StyleReader;
}
