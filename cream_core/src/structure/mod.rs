use std::iter::Empty;

use crate::{
    element::render_content::WildRenderContent, style::reader::StyleReader, CacheBox, Result,
};

use self::chain::Chain;

use crate::element::Element;

pub use self::{
    add_child::add_child,
    branch::Branch,
    repeating::Repeating,
    slot::{ApplySlot, Slot},
};

pub mod add_child;
pub mod branch;
pub mod cache_box;
pub mod chain;
mod fn_helper;
pub mod repeating;
pub mod slot;

pub trait Node {
    type Cache: Default + 'static;
    type StyleIter<'a, S>: Iterator<Item = S>
    where
        Self: 'a;

    fn style_iter<S>(&self) -> Self::StyleIter<'_, S>
    where
        S: StyleReader;

    /*fn finish<S, F, G>(self, cache: &mut Self::Cache, map: F) -> Result<()>
    where
        F: for<'r> FuncOnce<'r, Self::StyleIter<'r, S>, G>,
        S: StyleReader,
        G: GlobalEventRegister;*/

    fn finish_iter<'a, I>(self, cache: &mut Self::Cache, iter: I) -> Result<()>
    where
        I: Iterator<Item = WildRenderContent<'a>>;

    fn finish(self, cache: &mut CacheBox, content: WildRenderContent) -> Result<()>
    where
        Self: Sized,
        Self::Cache: Send + Sync,
    {
        self.finish_iter(cache.get_cache(), std::iter::once(content))
    }

    fn chain<T>(self, other: T) -> Chain<T, Self>
    where
        Self: Sized,
    {
        Chain(other, self)
    }
}

#[derive(Clone, Copy)]
pub struct EmptyStructure;

impl Node for EmptyStructure {
    type Cache = ();
    type StyleIter<'a, S> = Empty<S>;

    fn style_iter<S>(&self) -> Empty<S>
    where
        S: StyleReader,
    {
        std::iter::empty()
    }

    fn finish_iter<'a, I>(self, _: &mut Self::Cache, _: I) -> Result<()>
    where
        I: Iterator<Item = WildRenderContent<'a>>,
    {
        Ok(())
    }
}
