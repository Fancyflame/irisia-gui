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
    activate::{
        ActivateUpdateArguments, ActivatedStructure, Renderable, Structure, Visit,
        __private::StructureBuilderPrivate,
    },
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
    content: BareContent<'a>,
}

impl<T> IntoRendering<'_, T>
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

    pub fn finish_with<L>(self, layouter: &mut L, equality_matters: bool) -> Result<bool>
    where
        T: Renderable<L>,
    {
        self.activated.update(ActivateUpdateArguments {
            offset: 0,
            cache: self.cache,
            bare_content: self.content,
            layouter,
            equality_matters,
        })
    }

    pub fn finish(self, region: Region, equality_matters: bool) -> Result<bool>
    where
        T: Renderable<LayoutOnce>,
    {
        self.finish_with(&mut LayoutOnce::new(region), equality_matters)
    }
}

pub trait StructureBuilder: Sized + StructureBuilderPrivate {
    type Activated: ActivatedStructure;

    fn into_rendering<'a, 'bdr>(
        self,
        content: &'a mut RenderContent<'_, 'bdr>,
    ) -> IntoRendering<'a, Self::Activated>;

    fn chain<T>(self, other: T) -> Chain<Self, T>
    where
        Self: Sized,
    {
        Chain(self, other)
    }
}

impl<T: Sized + Structure> StructureBuilder for T {
    type Activated = <Self as Structure>::Activated;

    fn into_rendering<'a, 'bdr>(
        self,
        content: &'a mut RenderContent<'_, 'bdr>,
    ) -> IntoRendering<'a, Self::Activated> {
        let cache_box = match content.cache_box_for_children.take() {
            Some(c) => c.get_cache(),
            None => {
                panic!("this render content has been used to render once");
            }
        };

        into_rendering_raw(self, cache_box, content.bare.downgrade_lifetime())
    }
}

pub(crate) fn into_rendering_raw<'a, T: Structure>(
    node: T,
    cache: &'a mut <T::Activated as ActivatedStructure>::Cache,
    content_for_children: BareContent<'a>,
) -> IntoRendering<'a, T::Activated> {
    IntoRendering {
        activated: node.activate(cache, &content_for_children),
        cache,
        content: content_for_children,
    }
}
