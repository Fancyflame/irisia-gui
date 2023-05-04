pub mod add_child;
pub mod branch;
pub mod cache_box;
pub mod chain;
pub mod empty;
mod node;
pub mod repeating;
pub mod slot;

use anyhow::anyhow;

use crate::{
    element::{render_content::BareContent, Element, RenderContent},
    primary::Region,
    style::reader::StyleReader,
    Result,
};

pub use self::{
    add_child::add_child, branch::Branch, empty::EmptyStructure, repeating::Repeating, slot::Slot,
};
use self::{
    chain::Chain,
    node::{BareContentWrapper, RenderingNode},
};

pub struct IntoRendering<'a, T: RenderingNode> {
    node: T,
    cache: &'a mut T::Cache,
    content: BareContentWrapper<'a>,
}

impl<'a, T> IntoRendering<'a, T>
where
    T: RenderingNode,
{
    pub fn children_count(&self) -> usize {
        self.node.element_count()
    }

    pub fn visit_iter<S, Prop>(&self) -> T::VisitIter<'_, S>
    where
        S: StyleReader,
        T: VisitIter<Prop>,
    {
        self.node.visit_iter()
    }

    pub fn finish_iter<S, F>(self, mut map: F) -> Result<()>
    where
        F: FnMut(S, (Option<u32>, Option<u32>)) -> Result<Region>,
        S: StyleReader,
    {
        self.node.finish(self.cache, self.content, &mut map)
    }

    pub fn finish(self, region: Region) -> Result<()> {
        let mut region = Some(region);
        self.finish_iter(move |(), _| {
            region
                .take()
                .ok_or_else(|| anyhow!("only one element can be rendered"))
        })
    }
}

pub trait VisitIter<Prop>: RenderingNode {
    type VisitIter<'a, S>: Iterator<Item = VisitItem<S, Prop>>
    where
        S: StyleReader,
        Self: 'a;

    fn visit_iter<S>(&self) -> Self::VisitIter<'_, S>
    where
        S: StyleReader;
}

#[derive(Clone)]
pub struct VisitItem<S, P> {
    pub style: S,
    pub request_size: (Option<u32>, Option<u32>),
    pub child_props: P,
}

impl<T: Sized + RenderingNode> StructureBuilder for T {}
pub trait StructureBuilder: Sized + RenderingNode {
    fn into_rendering<'a>(self, content: &'a mut RenderContent) -> IntoRendering<'a, Self> {
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
    mut node: T,
    cache: &'a mut T::Cache,
    content_for_children: BareContent<'a>,
) -> IntoRendering<'a, T> {
    let wrapper = BareContentWrapper(content_for_children);

    node.prepare_for_rendering(cache, &wrapper);

    IntoRendering {
        node,
        cache,
        content: wrapper,
    }
}
