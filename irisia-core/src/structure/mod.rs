pub(crate) mod cache_box;
pub(crate) mod layer;
pub mod node;
pub mod visit;

use crate::{
    element::{render_content::BareContent, Element, RenderContent},
    primitive::Region,
    Result,
};

pub use self::{
    node::{
        add_child::{add_child, AddChild},
        branch::Branch,
        chain::Chain,
        empty::EmptyStructure,
        repeating::Repeating,
    },
    visit::{Layouter, VisitItem, Visitor},
};

use self::{
    __one_child::OnceLayouter,
    visit::{ActivatedStructure, BareContentWrapper, Renderable, Structure, Visit},
};

// IntoRendering

#[must_use]
pub struct IntoRendering<'a, T: ActivatedStructure> {
    activated: T,
    cache: &'a mut T::Cache,
    content: BareContentWrapper<'a>,
}

mod __one_child {
    use anyhow::anyhow;

    use crate::style::StyleContainer;

    use super::{visit::Layouter, *};
    pub struct OnceLayouter(pub Option<Region>);
    impl<El> Layouter<El> for OnceLayouter {
        fn layout(&mut self, _: VisitItem<El, impl StyleContainer>) -> Result<Region> {
            self.0
                .take()
                .ok_or_else(|| anyhow!("at most 1 element can be rendered"))
        }
    }
}

impl<'a, T> IntoRendering<'a, T>
where
    T: ActivatedStructure,
{
    pub fn children_count(&self) -> usize {
        self.activated.element_count()
    }

    pub fn visit<V>(&self, visitor: &mut V)
    where
        T: Visit<V>,
    {
        self.activated.visit(visitor)
    }

    pub fn finish_layouted<L>(self, layouter: &mut L) -> Result<()>
    where
        T: Renderable<L>,
    {
        self.activated.render(self.cache, self.content, layouter)
    }

    pub fn finish(self, region: Region) -> Result<()>
    where
        T: Renderable<OnceLayouter>,
    {
        self.finish_layouted(&mut OnceLayouter(Some(region)))
    }
}

// fn into_rendering

impl<T: Sized + Structure> StructureBuilder for T {}
pub trait StructureBuilder: Sized + Structure {
    fn into_rendering<'a>(
        self,
        content: &'a mut RenderContent,
    ) -> IntoRendering<'a, Self::Activated> {
        let cache_box = match content.cache_box_for_children.take() {
            Some(c) => c.get_cache(),
            None => {
                panic!("this render content has been used to render once");
            }
        };

        into_rendering_raw(self, cache_box, content.bare.downgrade_lifetime())
    }

    fn chain<T>(self, other: T) -> Chain<Self, T>
    where
        Self: Sized,
    {
        Chain(self, other)
    }
}

pub(crate) fn into_rendering_raw<'a, T: StructureBuilder>(
    node: T,
    cache: &'a mut <T::Activated as ActivatedStructure>::Cache,
    content_for_children: BareContent<'a>,
) -> IntoRendering<'a, T::Activated> {
    let wrapper = BareContentWrapper(content_for_children);
    IntoRendering {
        activated: node.activate(cache, &wrapper),
        cache,
        content: wrapper,
    }
}
