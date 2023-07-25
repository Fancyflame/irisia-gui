use std::marker::PhantomData;

use crate::{
    application::content::GlobalContent,
    element::{ComputeSize, SelfCache},
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

pub struct CacheUpdateArguments<'a, T, A> {
    pub(super) offset: usize,
    pub(super) cache: &'a mut T,
    pub(super) global_content: GlobalContent<'a>,
    pub(super) layouter: &'a mut A,
    pub(super) equality_matters: bool,
}

pub trait Structure: Sized {
    type Activated: ActivatedStructure;

    fn activate(self, cache: &mut SelfCache<Self>) -> Self::Activated;
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

pub trait UpdateCache<A>: ActivatedStructure + Sized {
    fn update(self, args: CacheUpdateArguments<Self::Cache, A>) -> Result<bool>;
}
