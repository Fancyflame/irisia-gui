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

pub trait VModel<'a> {
    const EXECUTE_POINTS: usize;
    type Storage: Model;

    fn create(self, dp: &mut DirtyPoints<'a>, ctx: &EMCreateCtx) -> Self::Storage;

    fn update(
        self,
        storage: &mut Self::Storage,
        dirty_points: &mut DirtyPoints<'a>,
        ctx: &EMCreateCtx,
    );
}

pub trait Model: 'static {
    fn visit(&self, f: &mut dyn FnMut(Element));
}

pub trait VNode<'a>: VModel<'a, Storage: GetElement> {}

impl<'a, T> VNode<'a> for T where T: VModel<'a, Storage: GetElement> {}

pub fn branch<'a, A, B>(b: Branch<A, B>) -> Branch<A, B>
where
    Branch<A, B>: VModel<'a>,
{
    b
}

pub fn repeat<'a, I>(iter: I) -> Repeat<I>
where
    Repeat<I>: VModel<'a>,
{
    Repeat { iter }
}

pub fn execute<'a, F>(f: F) -> Execute<F>
where
    Execute<F>: VModel<'a>,
{
    Execute { updator: f }
}
