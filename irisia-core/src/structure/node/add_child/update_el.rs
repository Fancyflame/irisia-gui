use std::marker::PhantomData;

use anyhow::Result;

use crate::{
    application::content::GlobalContent,
    element::SelfCache,
    primitive::Region,
    structure::{
        activate::{Structure, UpdateCache},
        layout_once::LayoutOnce,
        TreeBuilder,
    },
};

pub struct UpdateElementContent<'a, Ch: Structure> {
    pub(super) phantom_children: PhantomData<Ch>,
    pub(super) children_cache: &'a mut SelfCache<Ch>,
    pub(super) content: GlobalContent<'a>,
    pub(super) equality_matters: &'a mut bool,
    pub(super) interact_region: &'a mut Option<Region>,
}

impl<'a, Ch> UpdateElementContent<'a, Ch>
where
    Ch: Structure,
{
    pub fn child(self, draw_region: Region, children: Ch) -> Result<UpdateElementContent<'a, ()>>
    where
        Ch::Activated: UpdateCache<LayoutOnce>,
    {
        self.children(children, |tb| tb.finish(draw_region))
    }

    pub fn children<F>(self, children: Ch, f: F) -> Result<UpdateElementContent<'a, ()>>
    where
        F: FnOnce(TreeBuilder<'_, Ch::Activated>) -> Result<bool>,
    {
        *self.equality_matters = f(TreeBuilder::new(
            children,
            self.children_cache,
            self.content.downgrade_lifetime(),
            *self.equality_matters,
        ))?;

        Ok(UpdateElementContent {
            phantom_children: PhantomData,
            children_cache: Box::leak(Box::new(())),
            content: self.content,
            equality_matters: self.equality_matters,
            interact_region: self.interact_region,
        })
    }

    pub fn equality_matters(&self) -> bool {
        *self.equality_matters
    }

    pub fn mark_changed(self) -> Self {
        *self.equality_matters = false;
        self
    }

    pub fn set_interact_region(self, region: Region) -> Self {
        *self.interact_region = Some(region);
        self
    }

    pub fn clear_interact_region(self) -> Self {
        self.interact_region.take();
        self
    }
}
