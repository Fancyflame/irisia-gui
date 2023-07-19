use std::marker::PhantomData;

use crate::{
    element::{render_content::BareContent, ComputeSize},
    primitive::Region,
    style::StyleContainer,
    Result,
};

pub(super) mod __private {
    use super::Structure;

    pub trait StructureBuilderPrivate {}
    impl<T: Structure> StructureBuilderPrivate for T {}
}

pub trait Visitor<El, Pr> {
    fn visit(&mut self, item: VisitItem<El, Pr, impl StyleContainer>);
}

#[derive(Clone, Copy)]
pub struct VisitItem<'a, El, Pr, S> {
    pub _el: PhantomData<El>,
    pub index: usize,
    pub props: &'a Pr,
    pub styles: &'a S,
    pub request_size: ComputeSize,
}

pub trait Layouter<El, Pr> {
    fn layout(&mut self, item: VisitItem<El, Pr, impl StyleContainer>) -> Result<Region>;
}

pub struct ActivateUpdateArguments<'a, T, A> {
    pub(super) offset: usize,
    pub(super) cache: &'a mut T,
    pub(super) bare_content: BareContent<'a>,
    pub(super) layouter: &'a mut A,
    pub(super) equality_matters: bool,
}

pub trait Structure: Sized {
    type Activated: ActivatedStructure;

    fn activate(
        self,
        cache: &mut <Self::Activated as ActivatedStructure>::Cache,
        content: &BareContent,
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

    fn visit_at(&self, offset: usize, visitor: &mut V);
}

pub trait Renderable<A>: ActivatedStructure + Sized {
    #[must_use]
    fn update(self, args: ActivateUpdateArguments<Self::Cache, A>) -> Result<bool>;
}
