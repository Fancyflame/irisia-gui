use crate::{
    element::{render_content::BareContent, Element, RenderContent},
    primitive::Region,
    Result,
};

pub use self::{
    activate::{Layouter, VisitItem, Visitor},
    node::{
        add_child::{add_child, AddChild},
        branch::Branch,
        chain::Chain,
        empty::EmptyStructure,
        repeating::Repeating,
    },
};

use self::{
    activate::{ActivatedStructure, BareContentWrapper, Renderable, Structure, Visit},
    layout_once::LayoutOnce,
};

pub mod activate;
pub(crate) mod cache_box;
pub(crate) mod layer;
pub mod layout_once;
pub mod node;

// IntoRendering

#[must_use = "`IntoRendering` does nothing without calling `finish` or `finish_with`"]
pub struct IntoRendering<'a, T: ActivatedStructure> {
    activated: T,
    cache: &'a mut T::Cache,
    content: BareContentWrapper<'a>,
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

    pub fn finish_with<L>(self, layouter: &mut L) -> Result<()>
    where
        T: Renderable<L>,
    {
        self.activated.render(self.cache, self.content, layouter)
    }

    pub fn finish(self, region: Region) -> Result<()>
    where
        T: Renderable<LayoutOnce>,
    {
        self.finish_with(&mut LayoutOnce::new(region))
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
