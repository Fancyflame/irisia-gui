use std::hash::Hash;

use branch::Branch;
use execute::Execute;
use repeat::Repeat;

use crate::{
    prim_element::EMCreateCtx,
    prim_element::{Element, GetElement},
};

use super::tools::DirtyPoints;

pub mod branch;
pub mod execute;
pub mod repeat;
pub mod tuple;

pub trait VModel {
    const EXECUTE_POINTS: usize;
    type Storage: Model;

    fn create(self, dp: &mut DirtyPoints, ctx: &EMCreateCtx) -> Self::Storage;

    fn update(self, storage: &mut Self::Storage, dirty_points: &mut DirtyPoints, ctx: &EMCreateCtx);
}

pub trait Model: 'static {
    fn visit(&self, f: &mut dyn FnMut(Element));
}

pub trait VNode: VModel<Storage: GetElement> {}

impl<T> VNode for T where T: VModel<Storage: GetElement> {}

pub fn branch<A, B>(b: Branch<A, B>) -> impl VModel
where
    A: VModel,
    B: VModel,
{
    b
}

pub fn repeat<I, F, K>(iter: I, key_fn: F) -> impl VModel
where
    I: Iterator,
    I::Item: VModel,
    K: Hash + Eq + Clone + 'static,
    F: Fn(&I::Item) -> K,
{
    Repeat { iter, key_fn }
}

pub fn execute<F, R>(f: F) -> impl VModel
where
    F: FnOnce() -> R,
    R: VModel,
{
    Execute::new(f)
}
