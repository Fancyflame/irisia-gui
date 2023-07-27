use crate::{
    application::content::GlobalContent,
    element::{Element, SelfCache},
    primitive::Region,
    Result,
};

pub use self::{
    activate::{Layouter, VisitItem, Visitor},
    node::{
        add_child::{add_child, AddChild},
        branch::Branch,
        chain::Chain,
        repeating::Repeating,
    },
};

use self::{
    activate::{ActivatedStructure, CacheUpdateArguments, Structure, UpdateCache, Visit},
    layout_once::LayoutOnce,
};

pub mod activate;
pub mod cache;
pub mod into_tree_builder;
pub(crate) mod layer;
pub mod layout_once;
pub mod node;

// IntoRendering

#[must_use = "`IntoRendering` does nothing without calling `finish` or `finish_with`"]
pub struct TreeBuilder<'a, T: ActivatedStructure> {
    activated: T,
    cache: &'a mut T::Cache,
    content: GlobalContent<'a>,
    equality_matters: bool,
}

impl<'a, T> TreeBuilder<'a, T>
where
    T: ActivatedStructure,
{
    pub(crate) fn new<S>(
        node: S,
        cache: &'a mut SelfCache<S>,
        global_content: GlobalContent<'a>,
        equality_matters: bool,
    ) -> Self
    where
        S: Structure<Activated = T>,
    {
        TreeBuilder {
            activated: node.activate(cache),
            cache,
            content: global_content,
            equality_matters,
        }
    }

    pub fn children_count(&self) -> usize {
        self.activated.element_count()
    }

    pub fn visit<V>(&self, visitor: &mut V)
    where
        T: Visit<V>,
    {
        self.activated.visit(visitor)
    }

    pub fn finish_with<L>(self, layouter: &mut L) -> Result<bool>
    where
        T: UpdateCache<L>,
    {
        self.activated.update(CacheUpdateArguments {
            offset: 0,
            cache: self.cache,
            global_content: self.content,
            layouter,
            equality_matters: self.equality_matters,
        })
    }

    pub fn finish(self, region: Region) -> Result<bool>
    where
        T: UpdateCache<LayoutOnce>,
    {
        self.finish_with(&mut LayoutOnce::new(region))
    }
}
