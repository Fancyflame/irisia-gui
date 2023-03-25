use anyhow::anyhow;

use crate::{
    element::render_content::WildRenderContent, primary::Region, style::reader::StyleReader,
    CacheBox, Result,
};

use self::chain::Chain;

use crate::element::Element;

pub use self::{
    add_child::add_child,
    branch::Branch,
    empty::EmptyStructure,
    repeating::Repeating,
    slot::{ApplySlot, Slot},
};

pub mod add_child;
pub mod branch;
pub mod cache_box;
pub mod chain;
pub mod empty;
mod fn_helper;
pub mod repeating;
pub mod slot;

pub trait Node: Sized {
    type Cache: Default + Send + Sync + 'static;
    type StyleIter<'a, S>: Iterator<Item = S>
    where
        Self: 'a;

    fn style_iter<S>(&self) -> Self::StyleIter<'_, S>
    where
        S: StyleReader;

    fn __finish_iter<S, F>(
        self,
        cache: &mut Self::Cache,
        content: WildRenderContent,
        map: &mut F,
    ) -> Result<()>
    where
        F: FnMut(S, Option<Region>) -> Result<Region>,
        S: StyleReader;

    fn finish_iter<S, F>(
        self,
        cache_box: &mut CacheBox,
        content: WildRenderContent,
        mut map: F,
    ) -> Result<()>
    where
        F: FnMut(S, Option<Region>) -> Result<Region>,
        S: StyleReader,
    {
        self.__finish_iter(cache_box.get_cache(), content, &mut map)
    }

    fn finish(
        self,
        cache_box: &mut CacheBox,
        content: WildRenderContent,
        region: Region,
    ) -> Result<()>
    where
        Self: Sized,
        Self::Cache: Send + Sync,
    {
        let mut region = Some(region);
        self.__finish_iter(cache_box.get_cache(), content, &mut move |(), _| {
            region
                .take()
                .ok_or_else(|| anyhow!("only one element can be rendered"))
        })
    }

    fn chain<T>(self, other: T) -> Chain<T, Self>
    where
        Self: Sized,
    {
        Chain(other, self)
    }
}
