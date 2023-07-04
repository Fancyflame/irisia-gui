use crate::{
    element::render_content::BareContent, primitive::Region, style::StyleContainer, Result,
};

pub(crate) use __private::*;

mod __private {
    use super::*;
    pub struct BareContentWrapper<'a>(pub(crate) BareContent<'a>);

    pub trait Structure: Sized {
        type Activated: ActivatedStructure;

        fn activate(
            self,
            cache: &mut <Self::Activated as ActivatedStructure>::Cache,
            content: &BareContentWrapper,
        ) -> Self::Activated;
    }

    pub trait ActivatedStructure {
        type Cache: Default + 'static;

        fn element_count(&self) -> usize;
    }

    pub trait Visit<V>: ActivatedStructure {
        fn visit(&self, visitor: &mut V) {
            self.visit_at(0, visitor)
        }

        fn visit_at(&self, index: usize, visitor: &mut V);
    }

    pub trait Renderable<A>: ActivatedStructure + Sized {
        fn render(
            self,
            cache: &mut Self::Cache,
            bare_content: BareContentWrapper,
            draw_region_assigner: &mut A,
        ) -> Result<()> {
            self.render_at(0, cache, bare_content, draw_region_assigner)
        }

        fn render_at(
            self,
            index: usize,
            cache: &mut Self::Cache,
            bare_content: BareContentWrapper,
            draw_region_assigner: &mut A,
        ) -> Result<()>;
    }
}

pub trait Visitor<El> {
    fn visit(&mut self, item: VisitItem<El, impl StyleContainer>);
}

#[derive(Clone, Copy)]
pub struct VisitItem<'a, El, S> {
    pub index: usize,
    pub element: &'a El,
    pub style: &'a S,
    pub request_size: (Option<u32>, Option<u32>),
}

pub trait Layouter<El> {
    fn layout(&mut self, item: VisitItem<El, impl StyleContainer>) -> Result<Region>;
}
