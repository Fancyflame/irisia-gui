use std::iter::Empty;

use crate::{event::global_register::GlobalEventRegister, style::reader::StyleReader, Result};

use self::chain::Chain;

use crate::element::{Element, RenderContent};

pub use self::{
    add_child::add_child,
    branch::Branch,
    repeating::Repeating,
    slot::{ApplySlot, Slot},
};

pub mod add_child;
pub mod branch;
pub mod chain;
pub mod repeating;
pub mod slot;

// TODO
pub trait FuncOnce<'a, In>
where
    In: 'a,
{
    type Output: Iterator<Item = RenderContent<'a>>;
    fn provide(self, style_iter: In) -> Self::Output;
}

impl<'a, F, In, Out> FuncOnce<'a, In> for F
where
    In: 'a,
    F: FnOnce(In) -> Out,
    Out: Iterator<Item = RenderContent<'a>>,
{
    type Output = Out;
    fn provide(self, style_iter: In) -> Out {
        self(style_iter)
    }
}

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

    fn finish<'a, I>(self, cache: &mut Self::Cache, iter: I) -> Result<()>
    where
        I: Iterator<Item = RenderContent<'a>>;

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

    fn finish<'a, I>(self, _: &mut Self::Cache, _: I) -> Result<()>
    where
        I: Iterator<Item = RenderContent<'a>>,
    {
        Ok(())
    }
}
